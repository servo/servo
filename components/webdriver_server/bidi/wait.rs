use std::collections::HashMap;

use webdriver_traits::ids::ResumeId;

use crate::bidi::{
    WebDriverBidiThread,
    modules::{
        CommandHandled, ResponseSent,
        script::{Disowned, Evaluated},
    },
};

pub(crate) trait Resumable {
    type Event;

    fn resume(self, this: &mut WebDriverBidiThread, event: Self::Event);

    /// Whether an event is expected. If the result is true,
    /// algorithm will be removed from the wait queue.
    fn expects(&self, _event: &Self::Event) -> bool {
        true
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#wait-queue>.
pub(crate) struct WaitQueue<T: Resumable>(HashMap<ResumeId, T>);

impl<T: Resumable> Default for WaitQueue<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// We use a [`WaitQueues`] to hold all [`WaitQueue<T>`] to avoid
/// mess in script fields.
macro_rules! define_wait_queues {
    ($name:ident, $($field:ident: $t:ty),*$(,)?) => {
        #[derive(Default)]
        pub(crate) struct $name {
            $(
                $field: WaitQueue<$t>,
            )*
        }
        $(
            impl AsQueueMut<$t> for $name {
                fn as_queue_mut(&mut self) -> &mut WaitQueue<$t> {
                    &mut self.$field
                }
            }
        )*
    };
}

define_wait_queues! {
    WaitQueues,
    response_sent: ResponseSent,
    command_handled: CommandHandled,
    handle_disowned: Disowned,
    function_called: Evaluated,
}

/// The [`WaitQueues`] needs to impl `AsQueueMut` for all queue type
/// it holds, so that the generic can be used in [`awaits`] and [`resume`].
/// This is automatically done with `define_wait_queues`.
pub(crate) trait AsQueueMut<T: Resumable> {
    fn as_queue_mut(&mut self) -> &mut WaitQueue<T>;
}

impl WebDriverBidiThread {
    /// Pause the execution of one algorithm until specified events.
    /// See <https://www.w3.org/TR/webdriver-bidi/#awaits>.
    pub(crate) fn awaits<T>(&mut self, resume_id: ResumeId, algorithm: T)
    where
        T: Resumable,
        WaitQueues: AsQueueMut<T>,
    {
        self.wait_queues
            .as_queue_mut()
            .0
            .insert(resume_id, algorithm);
    }

    /// Resume an algorithem when specified events happened.
    /// See <https://www.w3.org/TR/webdriver-bidi/#resume>.
    pub(crate) fn resume<T>(&mut self, resume_id: ResumeId, event: T::Event)
    where
        T: Resumable,
        WaitQueues: AsQueueMut<T>,
    {
        let wait_queue = self.wait_queues.as_queue_mut();
        // Step 1. if no id, return
        let Some(algorithm) = wait_queue.0.get(&resume_id) else {
            return;
        };
        // Step 2 & 3. if the event is expected
        if algorithm.expects(&event) {
            // Step 3.1. remove event
            let algorithm = wait_queue.0.remove(&resume_id).unwrap();
            // Step 3.2. resume algorithm
            algorithm.resume(self, event);
        }
    }
}
