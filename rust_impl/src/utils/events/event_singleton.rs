use std::{
    sync::{
        Mutex, MutexGuard,
        mpsc::{self, Receiver, Sender},
    },
    time::Instant,
};

use super::app_events::{AppError, AppEvent};

static SUBSCRIBERS: Mutex<Vec<Sender<AppEvent>>> = Mutex::new(Vec::new());
static NUM_SUBSCRIBERS: Mutex<usize> = Mutex::new(0);

pub fn subscribe() -> Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();
    SUBSCRIBERS.lock().unwrap().push(tx);
    *NUM_SUBSCRIBERS.lock().unwrap() += 1;
    rx
}

pub fn shutdown() {
    let mut subscribers = SUBSCRIBERS.lock().unwrap();
    subscribers.clear();

    *NUM_SUBSCRIBERS.lock().unwrap() = 0;
}

pub fn emit(event: AppEvent) {
    if *NUM_SUBSCRIBERS.lock().unwrap() == 0 {
        return;
    }

    let mut subscribers: MutexGuard<'_, Vec<Sender<AppEvent>>> = SUBSCRIBERS.lock().unwrap();
    // retain keeps only the senders whose send() succeeded
    subscribers.retain(|tx| tx.send(event.clone()).is_ok());
}

pub fn emit_error(error: AppError) {
    emit(AppEvent::Error(error));
}

pub fn emit_timed_perf(event_label: &'static str, is_start: bool) {
    emit(AppEvent::PerfTimer {
        event_label,
        exact_instant: Instant::now(),
        is_start,
    });
}
