//! Time systems - now handled by Flecs systems in systems.rs
//!
//! The actual time update logic is done directly in the system definitions:
//! - TickWorldTime: calls WorldTime::tick()
//! - UpdateTps: calls TpsTracker::update(delta_time)
