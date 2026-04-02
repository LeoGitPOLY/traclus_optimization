use std::sync::{
    Mutex, MutexGuard,
    mpsc::{self, Receiver, Sender},
};

use crate::gui::app_events::{AppError, AppEvent};

static SUBSCRIBERS: Mutex<Vec<Sender<AppEvent>>> = Mutex::new(Vec::new());
static NUM_SUBSCRIBERS: Mutex<usize> = Mutex::new(0);

pub fn subscribe() -> Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();
    SUBSCRIBERS.lock().unwrap().push(tx);
    *NUM_SUBSCRIBERS.lock().unwrap() += 1;
    rx
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
