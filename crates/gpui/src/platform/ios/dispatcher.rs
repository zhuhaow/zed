use crate::{PlatformDispatcher, TaskLabel};
use async_task::Runnable;
use dispatch2::{run_on_main, Queue};
use objc2_foundation::NSThread;
use parking::{Parker, Unparker};
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};

pub(crate) struct IosDispatcher {
    parker: Arc<Mutex<Parker>>,
}

impl Default for IosDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl IosDispatcher {
    pub fn new() -> Self {
        IosDispatcher {
            parker: Arc::new(Mutex::new(Parker::new())),
        }
    }
}

impl PlatformDispatcher for IosDispatcher {
    fn is_main_thread(&self) -> bool {
        NSThread::isMainThread_class()
    }

    fn dispatch(&self, runnable: Runnable, _: Option<TaskLabel>) {
        Queue::global_queue(dispatch2::GlobalQueueIdentifier::Priority(
            dispatch2::QueuePriority::High,
        ))
        .exec_async(|| {
            runnable.run();
        });
    }

    fn dispatch_on_main_thread(&self, runnable: Runnable) {
        run_on_main(|_| {
            runnable.run();
        });
    }

    fn dispatch_after(&self, duration: Duration, runnable: Runnable) {
        Queue::global_queue(dispatch2::GlobalQueueIdentifier::Priority(
            dispatch2::QueuePriority::High,
        ))
        .after(duration, || {
            runnable.run();
        });
    }

    fn park(&self, timeout: Option<Duration>) -> bool {
        if let Some(timeout) = timeout {
            self.parker.lock().park_timeout(timeout)
        } else {
            self.parker.lock().park();
            true
        }
    }

    fn unparker(&self) -> Unparker {
        self.parker.lock().unparker()
    }
}
