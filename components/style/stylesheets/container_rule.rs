/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@container`][container] rule.
//!
//! [container]: https://drafts.csswg.org/css-contain-3/#container-rule

use crate::media_queries::MediaCondition;
use crate::shared_lock::{
    DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard,
};
use crate::values::specified::ContainerName;
use crate::str::CssStringWriter;
use crate::stylesheets::CssRules;
use cssparser::SourceLocation;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A container rule.
#[derive(Debug, ToShmem)]
pub struct ContainerRule {
    /// The container name.
    pub name: ContainerName,
    /// The container query.
    pub condition: ContainerCondition,
    /// The nested rules inside the block.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this rule was found.
    pub source_location: SourceLocation,
}

impl ContainerRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops)
            + self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl DeepCloneWithLock for ContainerRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        let rules = self.rules.read_with(guard);
        Self {
            name: self.name.clone(),
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock, guard, params))),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ContainerRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@container ")?;
        {
            let mut writer = CssWriter::new(dest);
            if !self.name.is_none() {
                self.name.to_css(&mut writer)?;
                writer.write_char(' ')?;
            }
            self.condition.to_css(&mut writer)?;
        }
        self.rules.read_with(guard).to_css_block(guard, dest)
    }
}

/// TODO: Factor out the media query code to work with containers.
pub type ContainerCondition = MediaCondition;
