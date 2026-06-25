/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use fonts::FontContext;
use net_traits::image_cache::FontResolver;
use resvg::usvg::{Font, fontdb};

pub struct SvgFontResolver {
    pub context: Arc<FontContext>,
}

impl FontResolver for SvgFontResolver {
    fn resolve(&self, _: &Font, _: &mut Arc<fontdb::Database>) -> Option<fontdb::ID> {
        None
    }
}
