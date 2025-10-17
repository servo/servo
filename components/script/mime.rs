/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use data_url::mime::Mime;
use headers::ContentType;

pub(crate) static APPLICATION: &str = "application";
pub(crate) static CHARSET: &str = "charset";
pub(crate) static HTML: &str = "html";
pub(crate) static TEXT: &str = "text";
pub(crate) static XML: &str = "xml";

/// Convenience methods to make the data_url Mime type more ergonomic.
pub(crate) trait MimeExt {
    /// Checks that the subtype has a given suffix.
    /// Eg. image/svg+xml has the xml suffix.
    fn has_suffix(&self, suffix: &str) -> bool;

    /// TODO: replace by a derive on data_url.
    fn clone(&self) -> Self;

    /// Build a Mime from the value of a Content-Type header.
    fn from_ct(ct: ContentType) -> Self;
}

impl MimeExt for Mime {
    fn has_suffix(&self, suffix: &str) -> bool {
        self.subtype.ends_with(&format!("+{}", suffix))
    }

    fn clone(&self) -> Self {
        Self {
            type_: self.type_.clone(),
            subtype: self.subtype.clone(),
            parameters: self.parameters.clone(),
        }
    }

    fn from_ct(ct: ContentType) -> Self {
        ct.to_string().parse().unwrap()
    }
}
