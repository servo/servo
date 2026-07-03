use std::{
    fmt::Display,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_atomic_id {
    ($name:ident, $atomic:ident) => {
        static $atomic: AtomicU64 = AtomicU64::new(0);

        #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        pub struct $name(u64);

        impl $name {
            pub fn next() -> Self {
                Self($atomic.fetch_add(1, Ordering::Relaxed))
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

macro_rules! define_uuid {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Default, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize,
        )]
        pub struct $name(Uuid);

        impl FromStr for $name {
            type Err = <Uuid as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(Self)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

define_atomic_id!(ConnectionId, CONNECTON_ID);
define_atomic_id!(ResumeId, RESUME_ID);

define_uuid!(HandleId);
define_uuid!(InternalId);
define_uuid!(PreloadScriptId);
define_uuid!(RealmId);
define_uuid!(SessionId);
define_uuid!(SubscriptionId);
