/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `@namespace` at-rule.

use crate::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::{Namespace, Prefix};
use cssparser::SourceLocation;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A `@namespace` rule.
#[derive(Clone, Debug, PartialEq, ToShmem)]
#[allow(missing_docs)]
pub struct NamespaceRule {
    /// The namespace prefix, and `None` if it's the default Namespace
    pub prefix: Option<Prefix>,
    /// The actual namespace url.
    pub url: Namespace,
    /// The source location this rule was found at.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for NamespaceRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSNamespaceRule
    fn to_css(
        &self,
        _guard: &SharedRwLockReadGuard,
        dest_str: &mut CssStringWriter,
    ) -> fmt::Result {
        let mut dest = CssWriter::new(dest_str);
        dest.write_str("@namespace ")?;
        if let Some(ref prefix) = self.prefix {
            prefix.to_css(&mut dest)?;
            dest.write_char(' ')?;
        }
        dest.write_str("url(")?;
        self.url.to_string().to_css(&mut dest)?;
        dest.write_str(");")
    }
}
