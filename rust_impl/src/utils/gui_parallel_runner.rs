// gui_parallel_runner.rs - Enforces one-task-at-a-time execution for GUI-triggered work on MainTraclusDL
//
// The GUI owns one GuiParallelRunner. Every button that triggers computation
// calls try_run(...). If a task is already running, try_run returns false
// immediately and the button stays disabled. Otherwise it spawns one std::thread
// which may internally use the custom Rayon pool inside MainTraclusDL.

use std::sync::{Arc, Mutex};
use std::thread;

use crate::algorithms::main_traclusdl::MainTraclusDL;

// Shared bool: true while any task is executing.
// Shared ownership between GUI thread and worker thread.
type RunningFlag = Arc<Mutex<bool>>;

// RAII guard: sets the flag back to false when it goes out of scope.
struct ReleaseOnDrop(RunningFlag);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = false;
    }
}

// ─────────────────────────────────────────────
// GuiParallelRunner
// ─────────────────────────────────────────────

pub struct GuiParallelRunner {
    is_running: RunningFlag,
}

impl GuiParallelRunner {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Generic task launcher.
    ///
    /// - `main`: Arc-wrapped MainTraclusDL shared with the worker thread
    /// - `task`: any fonction on `&mut MainTraclusDL`; runs on a dedicated std::thread
    ///
    /// Returns false immediately (no blocking) if already busy.
    /// Returns true if the task was accepted and spawned.
    pub fn try_run<F>(&self, main: Arc<Mutex<MainTraclusDL>>, task: F) -> bool
    where
        F: FnOnce(&mut MainTraclusDL) + Send + 'static,
    {
        if !self.try_acquire() {
            return false;
        }

        let flag = Arc::clone(&self.is_running);

        thread::spawn(move || {
            // Guard releases the flag when this thread scope exits, even on panic
            let _guard = ReleaseOnDrop(flag);
            task(&mut main.lock().unwrap());
        });

        true
    }

    // Atomically checks and sets the flag.
    // Returns false if already running, true if successfully acquired.
    fn try_acquire(&self) -> bool {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return false;
        }
        *running = true;
        true
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
}
