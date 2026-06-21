use std::rc::Rc;

use log::warn;
use servo_base::id::BrowsingContextId;
use webdriver_traits::bidi::{Event, EventData};

use crate::bidi::{remote_end::RemoteEnd, session::SessionId};

impl RemoteEnd {
    /// Remote end subscribe steps for `log.entryAdded`.
    pub(crate) async fn subscribe_log_entry_added(
        self: Rc<Self>,
        session_id: SessionId,
        navigable_ids: &[BrowsingContextId],
        include_global: bool,
    ) {
        let log_event_buffer = {
            let active_sessions = self.active_sessions.borrow();
            let Some(log_event_buffer) = active_sessions
                .get(&session_id)
                .map(|s| &s.log_event_buffer)
            else {
                warn!("The session {session_id:?} to subscribe log.entryAdded is not active");
                return;
            };
            log_event_buffer.replace(Default::default())
        };
        // Step 1.
        for (navigable_id, events) in log_event_buffer.iter() {
            // Step 1.1.
            let maybe_context = self.get_a_navigable(*navigable_id);
            let navigable = match maybe_context {
                // Step 1.2. skip as remove is not needed
                Err(_) => continue,
                // Step 1.3.
                Ok(navigable) => navigable,
            };
            // Step 1.4.
            // TODO: further get top id
            let top_level_traversable = navigable.traversable_id;
            let top_level_navigable = self
                .traversables
                .borrow()
                .get(&top_level_traversable)
                // XXX: unwrap, [0] is bad
                .unwrap()
                .navigables[0];
            // Step 1.5.
            if let (true, false) | (false, true) =
                (include_global, navigable_ids.contains(&top_level_navigable))
            {
                let mut removed_events = vec![];
                // Step 1.5.1.
                for (event, other_navigables) in events.iter() {
                    if removed_events.contains(&event) {
                        continue;
                    }
                    // Step 1.5.1.1.
                    self.clone()
                        .emit_an_event(
                            session_id,
                            Event {
                                event_data: EventData::LogEvent(event.clone()),
                                extensible: Default::default(),
                            },
                        )
                        .await;
                    // Step 1.5.1.2. mark duplicate event removed
                    if !other_navigables.is_empty() {
                        removed_events.push(event);
                    }
                }
            }
        }
    }
}
