//! Dynamic plugin loader for hot-reloadable Flecs modules
//!
//! This crate provides functionality to load, unload, and hot-reload
//! Rust dylib plugins that export Flecs modules.
//!
//! # Plugin Interface
//!
//! Each plugin dylib must export these symbols:
//! - `plugin_load(world: &World)` - Called to load/register the module
//! - `plugin_unload(world: &World)` - Called before unloading to cleanup
//! - `plugin_name() -> &'static str` - Returns the plugin name
//!
//! # Safety
//!
//! This loader assumes all plugins are compiled with the same Rust version
//! and toolchain. ABI compatibility is NOT guaranteed across different
//! Rust versions.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use flecs_ecs::prelude::World;
use libloading::{Library, Symbol};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Plugin function signatures
type PluginLoadFn = extern "Rust" fn(&World);
type PluginUnloadFn = extern "Rust" fn(&World);
type PluginNameFn = extern "Rust" fn() -> &'static str;

/// Errors that can occur during plugin operations
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to load library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    #[error("Missing required symbol '{symbol}' in plugin")]
    MissingSymbol { symbol: &'static str },

    #[error("Plugin not found: {0}")]
    NotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),
}

/// A loaded plugin instance
struct LoadedPlugin {
    /// The loaded dynamic library
    library: Library,
    /// Path to the plugin file
    path: PathBuf,
    /// Plugin name (cached from plugin_name())
    name: String,
}

impl LoadedPlugin {
    /// Load a plugin from the given path
    ///
    /// # Safety
    /// The plugin must be compiled with the same Rust version as the loader.
    unsafe fn load(path: &Path) -> Result<Self, PluginError> {
        debug!("Loading plugin from: {}", path.display());

        let library = unsafe { Library::new(path)? };

        // Get plugin name
        let name_fn: Symbol<'_, PluginNameFn> = unsafe {
            library
                .get(b"plugin_name")
                .map_err(|_| PluginError::MissingSymbol {
                    symbol: "plugin_name",
                })?
        };
        let name = name_fn().to_string();

        info!("Loaded plugin '{}' from {}", name, path.display());

        Ok(Self {
            library,
            path: path.to_path_buf(),
            name,
        })
    }

    /// Initialize the plugin by calling plugin_load
    fn init(&self, world: &World) -> Result<(), PluginError> {
        debug!("Initializing plugin '{}'", self.name);

        let load_fn: Symbol<'_, PluginLoadFn> = unsafe {
            self.library
                .get(b"plugin_load")
                .map_err(|_| PluginError::MissingSymbol {
                    symbol: "plugin_load",
                })?
        };

        load_fn(world);
        info!("Initialized plugin '{}'", self.name);
        Ok(())
    }

    /// Cleanup the plugin by calling plugin_unload
    fn cleanup(&self, world: &World) -> Result<(), PluginError> {
        debug!("Cleaning up plugin '{}'", self.name);

        let unload_fn: Symbol<'_, PluginUnloadFn> = unsafe {
            self.library
                .get(b"plugin_unload")
                .map_err(|_| PluginError::MissingSymbol {
                    symbol: "plugin_unload",
                })?
        };

        unload_fn(world);
        info!("Cleaned up plugin '{}'", self.name);
        Ok(())
    }
}

/// Plugin loader and manager
pub struct PluginLoader {
    /// Directory to scan for plugins
    plugins_dir: PathBuf,
    /// Currently loaded plugins (keyed by file path)
    plugins: HashMap<PathBuf, LoadedPlugin>,
    /// File watcher for hot-reload
    watcher: Option<RecommendedWatcher>,
    /// Channel for file change events
    watch_rx: Option<mpsc::Receiver<Result<Event, notify::Error>>>,
}

impl PluginLoader {
    /// Create a new plugin loader for the given directory
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            plugins: HashMap::new(),
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

    /// Scan the plugins directory and load all plugins
    pub fn load_all(&mut self, world: &World) -> Result<(), PluginError> {
        let ext = Self::dylib_extension();
        info!(
            "Scanning for plugins in: {} (*.{})",
            self.plugins_dir.display(),
            ext
        );

        if !self.plugins_dir.exists() {
            warn!(
                "Plugins directory does not exist: {}",
                self.plugins_dir.display()
            );
            std::fs::create_dir_all(&self.plugins_dir)?;
            info!("Created plugins directory");
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.plugins_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() == Some(OsStr::new(ext))
                && let Err(e) = self.load_plugin(&path, world)
            {
                error!("Failed to load plugin {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Load a single plugin from the given path
    pub fn load_plugin(&mut self, path: &Path, world: &World) -> Result<(), PluginError> {
        // Unload existing plugin at this path if any
        if self.plugins.contains_key(path) {
            self.unload_plugin(path, world)?;
        }

        let plugin = unsafe { LoadedPlugin::load(path)? };
        plugin.init(world)?;
        self.plugins.insert(path.to_path_buf(), plugin);

        Ok(())
    }

    /// Unload a plugin at the given path
    pub fn unload_plugin(&mut self, path: &Path, world: &World) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.remove(path) {
            plugin.cleanup(world)?;
            // Library is dropped here, unloading the dylib
            info!("Unloaded plugin '{}' from {}", plugin.name, path.display());
        }
        Ok(())
    }

    /// Reload a plugin (unload then load)
    pub fn reload_plugin(&mut self, path: &Path, world: &World) -> Result<(), PluginError> {
        info!("Reloading plugin: {}", path.display());
        self.unload_plugin(path, world)?;

        // Small delay to ensure file is fully written
        std::thread::sleep(std::time::Duration::from_millis(100));

        self.load_plugin(path, world)?;
        Ok(())
    }

    /// Start watching the plugins directory for changes
    pub fn start_watching(&mut self) -> Result<(), PluginError> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )?;

        watcher.watch(&self.plugins_dir, RecursiveMode::NonRecursive)?;

        info!(
            "Started watching plugins directory: {}",
            self.plugins_dir.display()
        );

        self.watcher = Some(watcher);
        self.watch_rx = Some(rx);

        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.watch_rx = None;
        info!("Stopped watching plugins directory");
    }

    /// Poll for file changes and reload modified plugins
    ///
    /// Call this each frame/tick to check for plugin updates.
    /// Returns the number of plugins reloaded.
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
                    debug!("Detected change in plugin: {}", path.display());
                    paths_to_reload.push(path);
                }
            }
        }

        // Now reload the plugins
        let mut reloaded = 0;
        for path in paths_to_reload {
            match self.reload_plugin(&path, world) {
                Ok(()) => reloaded += 1,
                Err(e) => error!("Failed to reload plugin {}: {}", path.display(), e),
            }
        }

        reloaded
    }

    /// Unload all plugins
    pub fn unload_all(&mut self, world: &World) {
        let paths: Vec<_> = self.plugins.keys().cloned().collect();
        for path in paths {
            if let Err(e) = self.unload_plugin(&path, world) {
                error!("Failed to unload plugin {}: {}", path.display(), e);
            }
        }
    }

    /// Get the list of loaded plugin names
    pub fn loaded_plugins(&self) -> Vec<&str> {
        self.plugins.values().map(|p| p.name.as_str()).collect()
    }
}

impl Drop for PluginLoader {
    fn drop(&mut self) {
        // Note: We can't unload plugins here because we don't have the world reference
        // The caller should call unload_all() before dropping the loader
        if !self.plugins.is_empty() {
            warn!(
                "PluginLoader dropped with {} plugins still loaded",
                self.plugins.len()
            );
        }
    }
}
