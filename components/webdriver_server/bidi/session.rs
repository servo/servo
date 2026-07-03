use std::collections::{HashMap, HashSet};

use bitflags::bitflags;
use servo_base::id::{BrowsingContextId, PipelineId};
use webdriver_traits::{
    bidi::script::ChannelValue,
    ids::{ConnectionId, PreloadScriptId, RealmId, SessionId, SubscriptionId},
};

pub(crate) struct Session {
    pub(crate) id: SessionId,
    pub(crate) flags: SessionFlags,
    pub(crate) connections: HashSet<ConnectionId>,
    pub(crate) preload_script_map: HashMap<PreloadScriptId, PreloadScriptMapValue>,
    // TODO: sandbox is not implemented
    /// See <https://www.w3.org/TR/webdriver-bidi/#sandbox-map>.
    #[allow(unused)]
    pub(crate) sandbox_map: HashMap<PipelineId, HashMap<String, RealmId>>,
    pub(crate) subscriptions: HashMap<SubscriptionId, Subscription>,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionFlags(u8);

bitflags! {
    impl SessionFlags: u8 {
        const HTTP = 0b001;
        const BIDI = 0b010;
        const CHAN = 0b100;
    }
}

pub(crate) struct PreloadScriptMapValue {
    pub(crate) function_declaration: String,
    pub(crate) arguments: Vec<ChannelValue>,
    pub(crate) navigables: Vec<BrowsingContextId>,
    pub(crate) sandbox: Option<String>,
    // TODO: user context is not implemented
    #[allow(unused)]
    pub(crate) user_contexts: Vec<String>,
    pub(crate) realms: Vec<RealmId>,
}

#[derive(Clone, Debug)]
pub(crate) struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) event_names: HashSet<String>,
    pub(crate) top_level_traversable_ids: HashSet<BrowsingContextId>,
    pub(crate) user_context_ids: HashSet<String>,
}

impl Session {
    /// See <https://www.w3.org/TR/webdriver-bidi/#event-is-enabled>.
    pub(crate) fn event_is_enabled(&self, event_name: &str) -> bool {
        // TODO: user context is not implemented, session checked only
        for subscription in self.subscriptions.values() {
            if subscription.event_names.contains(event_name) {
                return true;
            }
        }
        false
    }
}

impl Subscription {
    /// <https://www.w3.org/TR/webdriver-bidi/#subscription-global>
    pub(crate) fn is_global(&self) -> bool {
        self.top_level_traversable_ids.is_empty() && self.user_context_ids.is_empty()
    }
}
