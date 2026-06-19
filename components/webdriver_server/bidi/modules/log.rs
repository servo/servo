use std::rc::Rc;

use log::warn;
use servo_base::id::BrowsingContextId;

use crate::bidi::{remote_end::RemoteEnd, session::common::SessionId};

impl RemoteEnd {
    pub(crate) fn subscribe_log_entry_added(
        self: Rc<Self>,
        session_id: SessionId,
        navigable_ids: &[BrowsingContextId],
        include_global: bool,
    ) {
        // TODO: refcell may be too large, when will there be borrow_mut active sessions?
        let active_sessions = self.active_sessions.borrow();
        let Some(log_event_buffer) = active_sessions
            .get(&session_id)
            .map(|s| &s.log_event_buffer)
        else {
            warn!(
                "The session to subscribe log.entryAdded is not active (id: {:?})",
                session_id
            );
            return;
        };
        // Step 1.
        for (navigable_id, events) in log_event_buffer.borrow().iter() {
            // Step 1.1.
            let maybe_context = self.get_a_navigable(*navigable_id);
            let navigable = match maybe_context {
                // Step 1.2.
                Err(_) => {
                    log_event_buffer.borrow_mut().remove(navigable_id);
                    continue;
                },
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
                // Step 1.5.1.
                // for (event, other_navigables) in events {
                //     // TODO: be careful, borrow across await here
                // // Step 1.5.1.1.
                // TODO: crazy mut in iteration, rc needed
                // }
            }
        }
    }
}
