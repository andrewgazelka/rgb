//! Dynamic module loader for hot-reloadable Flecs modules
//!
//! This crate provides functionality to load, unload, and hot-reload
//! Rust dylib modules that export Flecs modules.
//!
//! # Module Interface
//!
//! Each module dylib must export these Rust ABI symbols:
//! - `module_load(world: &World)` - Called to load/register the module
//! - `module_unload(world: &World)` - Called before unloading to cleanup
//! - `module_name() -> &'static str` - Returns the module name
//! - `module_version() -> u32` - (optional) Returns the module version
//!
//! # Using the `register_module!` macro
//!
//! Instead of manually writing the module exports, use the macro:
//!
//! ```ignore
//! use module_loader::register_module;
//!
//! register_module! {
//!     name: "my-module",
//!     version: 1,
//!     module: MyModule,
//!     path: "::my_module",
//! }
//! ```
//!
//! # Safety
//!
//! Modules use Rust ABI which requires the same compiler version.
//! Both host and modules must link to the same `libflecs_ecs.dylib`.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use flecs_ecs::prelude::World;
#[cfg(unix)]
use libloading::os::unix::{Library, Symbol};
#[cfg(windows)]
use libloading::{Library, Symbol};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Ensure flecs_ecs shared library is loaded with RTLD_GLOBAL on Unix.
/// This must be called before loading any modules that depend on flecs_ecs.
#[cfg(unix)]
pub fn ensure_flecs_global() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Find libflecs_ecs.dylib/.so in the same directory as the executable
        // or in the standard library paths
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let lib_name = if cfg!(target_os = "macos") {
            "libflecs_ecs.dylib"
        } else {
            "libflecs_ecs.so"
        };

        // Try paths in order: exe/../deps, exe dir, then let system find it
        let paths_to_try: Vec<PathBuf> = [
            exe_dir.as_ref().map(|d| d.join("deps").join(lib_name)),
            exe_dir.as_ref().map(|d| d.join(lib_name)),
            Some(PathBuf::from(lib_name)),
        ]
        .into_iter()
        .flatten()
        .collect();

        for path in &paths_to_try {
            if path.exists() || path.to_str() == Some(lib_name) {
                debug!("Trying to load flecs_ecs from: {}", path.display());
                let result = unsafe {
                    Library::open(Some(path), libc::RTLD_NOW | libc::RTLD_GLOBAL)
                };
                match result {
                    Ok(lib) => {
                        info!("Loaded flecs_ecs with RTLD_GLOBAL from: {}", path.display());
                        // Intentionally leak the library so it stays loaded
                        std::mem::forget(lib);
                        return;
                    }
                    Err(e) => {
                        debug!("Failed to load from {}: {}", path.display(), e);
                    }
                }
            }
        }

        warn!("Could not load libflecs_ecs with RTLD_GLOBAL - module loading may fail");
    });
}

#[cfg(not(unix))]
pub fn ensure_flecs_global() {
    // Windows handles symbol visibility differently
}

/// Module function signatures (Rust ABI - requires same compiler version)
type ModuleLoadFn = fn(&World);
type ModuleUnloadFn = fn(&World);
type ModuleNameFn = fn() -> &'static str;
type ModuleVersionFn = fn() -> u32;

/// Errors that can occur during module operations
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Failed to load library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    #[error("Missing required symbol '{symbol}' in module")]
    MissingSymbol { symbol: &'static str },

    #[error("Module not found: {0}")]
    NotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),
}

/// A loaded module instance
struct LoadedModule {
    /// The loaded dynamic library
    library: Library,
    /// Path to the module file
    path: PathBuf,
    /// Module name (cached from module_name())
    name: String,
    /// Module version (optional, from module_version())
    version: Option<u32>,
}

impl LoadedModule {
    /// Load a module from the given path
    ///
    /// # Safety
    /// The module must be compiled with the same Rust version as the loader.
    #[cfg(unix)]
    unsafe fn load(path: &Path) -> Result<Self, ModuleError> {
        debug!("Loading module from: {}", path.display());

        // Use RTLD_NOW | RTLD_GLOBAL so symbols are available to other modules
        // This is essential for modules to share the same flecs_ecs symbols
        let library = unsafe {
            Library::open(Some(path), libc::RTLD_NOW | libc::RTLD_GLOBAL)?
        };

        Self::load_inner(library, path)
    }

    /// Load a module from the given path (Windows)
    ///
    /// # Safety
    /// The module must be compiled with the same Rust version as the loader.
    #[cfg(windows)]
    unsafe fn load(path: &Path) -> Result<Self, ModuleError> {
        debug!("Loading module from: {}", path.display());

        let library = unsafe { Library::new(path)? };

        Self::load_inner(library, path)
    }

    /// Common loading logic after library is opened
    fn load_inner(library: Library, path: &Path) -> Result<Self, ModuleError> {

        // Get module name
        let name_fn: Symbol<ModuleNameFn> = unsafe {
            library
                .get(b"module_name")
                .map_err(|_| ModuleError::MissingSymbol {
                    symbol: "module_name",
                })?
        };
        let name = name_fn();

        // Try to get version (optional)
        let version = unsafe { library.get::<ModuleVersionFn>(b"module_version") }
            .ok()
            .map(|f| f());

        if let Some(v) = version {
            info!("Loaded module '{}' v{} from {}", name, v, path.display());
        } else {
            info!("Loaded module '{}' from {}", name, path.display());
        }

        Ok(Self {
            library,
            path: path.to_path_buf(),
            name: name.to_string(),
            version,
        })
    }

    /// Initialize the module by calling module_load
    fn init(&self, world: &World) -> Result<(), ModuleError> {
        debug!("Initializing module '{}'", self.name);

        let load_fn: Symbol<ModuleLoadFn> = unsafe {
            self.library
                .get(b"module_load")
                .map_err(|_| ModuleError::MissingSymbol {
                    symbol: "module_load",
                })?
        };

        load_fn(world);
        info!("Initialized module '{}'", self.name);
        Ok(())
    }

    /// Cleanup the module by calling module_unload
    fn cleanup(&self, world: &World) -> Result<(), ModuleError> {
        debug!("Cleaning up module '{}'", self.name);

        let unload_fn: Symbol<ModuleUnloadFn> = unsafe {
            self.library
                .get(b"module_unload")
                .map_err(|_| ModuleError::MissingSymbol {
                    symbol: "module_unload",
                })?
        };

        unload_fn(world);
        info!("Cleaned up module '{}'", self.name);
        Ok(())
    }
}

/// Module loader and manager
pub struct ModuleLoader {
    /// Directory to scan for modules
    modules_dir: PathBuf,
    /// Currently loaded modules (keyed by file path)
    modules: HashMap<PathBuf, LoadedModule>,
    /// File watcher for hot-reload
    watcher: Option<RecommendedWatcher>,
    /// Channel for file change events
    watch_rx: Option<mpsc::Receiver<Result<Event, notify::Error>>>,
}

impl ModuleLoader {
    /// Create a new module loader for the given directory
    pub fn new(modules_dir: impl Into<PathBuf>) -> Self {
        Self {
            modules_dir: modules_dir.into(),
            modules: HashMap::new(),
            watcher: None,
            watch_rx: None,
        }
    }

    /// Get the platform-specific dynamic library extension
    fn dylib_extension() -> &'static str {
        if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "windows") {
            "dll"
        } else {
            "so"
        }
    }

    /// Scan the modules directory and load all modules
    pub fn load_all(&mut self, world: &World) -> Result<(), ModuleError> {
        // Ensure flecs_ecs is loaded with RTLD_GLOBAL before loading any modules
        ensure_flecs_global();

        let ext = Self::dylib_extension();
        info!(
            "Scanning for modules in: {} (*.{})",
            self.modules_dir.display(),
            ext
        );

        if !self.modules_dir.exists() {
            warn!(
                "Modules directory does not exist: {}",
                self.modules_dir.display()
            );
            std::fs::create_dir_all(&self.modules_dir)?;
            info!("Created modules directory");
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.modules_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() == Some(OsStr::new(ext))
                && let Err(e) = self.load_module(&path, world)
            {
                error!("Failed to load module {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Load a single module from the given path
    pub fn load_module(&mut self, path: &Path, world: &World) -> Result<(), ModuleError> {
        // Unload existing module at this path if any
        if self.modules.contains_key(path) {
            self.unload_module(path, world)?;
        }

        let module = unsafe { LoadedModule::load(path)? };
        module.init(world)?;
        self.modules.insert(path.to_path_buf(), module);

        Ok(())
    }

    /// Unload a module at the given path
    pub fn unload_module(&mut self, path: &Path, world: &World) -> Result<(), ModuleError> {
        if let Some(module) = self.modules.remove(path) {
            module.cleanup(world)?;
            // Library is dropped here, unloading the dylib
            info!("Unloaded module '{}' from {}", module.name, path.display());
        }
        Ok(())
    }

    /// Reload a module (unload then load)
    pub fn reload_module(&mut self, path: &Path, world: &World) -> Result<(), ModuleError> {
        info!("Reloading module: {}", path.display());
        self.unload_module(path, world)?;

        // Small delay to ensure file is fully written
        std::thread::sleep(std::time::Duration::from_millis(100));

        self.load_module(path, world)?;
        Ok(())
    }

    /// Start watching the modules directory for changes
    pub fn start_watching(&mut self) -> Result<(), ModuleError> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )?;

        watcher.watch(&self.modules_dir, RecursiveMode::NonRecursive)?;

        info!(
            "Started watching modules directory: {}",
            self.modules_dir.display()
        );

        self.watcher = Some(watcher);
        self.watch_rx = Some(rx);

        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.watch_rx = None;
        info!("Stopped watching modules directory");
    }

    /// Poll for file changes and reload modified modules
    ///
    /// Call this each frame/tick to check for module updates.
    /// Returns the number of modules reloaded.
    pub fn poll_reload(&mut self, world: &World) -> usize {
        let Some(rx) = &self.watch_rx else {
            return 0;
        };

        let ext = Self::dylib_extension();

        // Collect paths to reload first (to avoid borrow issues)
        let mut paths_to_reload = Vec::new();

        // Process all pending events
        while let Ok(event_result) = rx.try_recv() {
            let Ok(event) = event_result else {
                continue;
            };

            // We care about modify and create events
            let dominated = matches!(
                event.kind,
                notify::EventKind::Modify(_) | notify::EventKind::Create(_)
            );

            if !dominated {
                continue;
            }

            for path in event.paths {
                if path.extension() == Some(OsStr::new(ext)) {
                    debug!("Detected change in module: {}", path.display());
                    paths_to_reload.push(path);
                }
            }
        }

        // Deduplicate paths (file watcher can send multiple events for same file)
        paths_to_reload.sort_unstable();
        paths_to_reload.dedup();

        // Now reload the modules
        let mut reloaded = 0;
        for path in paths_to_reload {
            match self.reload_module(&path, world) {
                Ok(()) => reloaded += 1,
                Err(e) => error!("Failed to reload module {}: {}", path.display(), e),
            }
        }

        reloaded
    }

    /// Reload all currently loaded modules
    pub fn reload_all(&mut self, world: &World) -> usize {
        let paths: Vec<_> = self.modules.keys().cloned().collect();
        let mut reloaded = 0;
        for path in paths {
            match self.reload_module(&path, world) {
                Ok(()) => reloaded += 1,
                Err(e) => error!("Failed to reload module {}: {}", path.display(), e),
            }
        }
        reloaded
    }

    /// Unload all modules
    pub fn unload_all(&mut self, world: &World) {
        let paths: Vec<_> = self.modules.keys().cloned().collect();
        for path in paths {
            if let Err(e) = self.unload_module(&path, world) {
                error!("Failed to unload module {}: {}", path.display(), e);
            }
        }
    }

    /// Get the list of loaded module names with versions
    pub fn loaded_modules(&self) -> Vec<String> {
        self.modules
            .values()
            .map(|p| {
                if let Some(v) = p.version {
                    format!("{} v{}", p.name, v)
                } else {
                    p.name.clone()
                }
            })
            .collect()
    }
}

impl Drop for ModuleLoader {
    fn drop(&mut self) {
        // Note: We can't unload modules here because we don't have the world reference
        // The caller should call unload_all() before dropping the loader
        if !self.modules.is_empty() {
            warn!(
                "ModuleLoader dropped with {} modules still loaded",
                self.modules.len()
            );
        }
    }
}

/// Register a Flecs module as a hot-reloadable module.
///
/// This macro generates the required `no_mangle` exports for the module loader.
///
/// # Example
///
/// ```ignore
/// use flecs_ecs::prelude::*;
/// use module_loader::register_module;
///
/// #[derive(Component)]
/// pub struct MyModule;
///
/// impl Module for MyModule {
///     fn module(world: &World) {
///         world.module::<MyModule>("my_module");
///         // ... register components and systems
///     }
/// }
///
/// register_module! {
///     name: "my-module",
///     version: 1,
///     module: MyModule,
///     path: "::my_module",
/// }
/// ```
#[macro_export]
macro_rules! register_module {
    {
        name: $name:literal,
        version: $version:expr,
        module: $module:ty,
        path: $path:literal $(,)?
    } => {
        #[unsafe(no_mangle)]
        pub fn module_load(world: &::flecs_ecs::prelude::World) {
            world.import::<$module>();
        }

        #[unsafe(no_mangle)]
        pub fn module_unload(world: &::flecs_ecs::prelude::World) {
            if let Some(e) = world.try_lookup($path) {
                e.destruct();
            }
        }

        #[unsafe(no_mangle)]
        pub fn module_name() -> &'static str {
            $name
        }

        #[unsafe(no_mangle)]
        pub fn module_version() -> u32 {
            $version
        }
    };
}
