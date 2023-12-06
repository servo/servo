/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A set of author stylesheets and their computed representation, such as the
//! ones used for ShadowRoot.

use crate::dom::TElement;
use crate::invalidation::media_queries::ToMediaListKey;
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheet_set::AuthorStylesheetSet;
use crate::stylesheets::StylesheetInDocument;
use crate::stylist::CascadeData;
use crate::stylist::Stylist;
use servo_arc::Arc;

/// A set of author stylesheets and their computed representation, such as the
/// ones used for ShadowRoot.
#[derive(MallocSizeOf)]
pub struct GenericAuthorStyles<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The sheet collection, which holds the sheet pointers, the invalidations,
    /// and all that stuff.
    pub stylesheets: AuthorStylesheetSet<S>,
    /// The actual cascade data computed from the stylesheets.
    #[ignore_malloc_size_of = "Measured as part of the stylist"]
    pub data: Arc<CascadeData>,
}

pub use self::GenericAuthorStyles as AuthorStyles;

lazy_static! {
    static ref EMPTY_CASCADE_DATA: Arc<CascadeData> = Arc::new_leaked(CascadeData::new());
}

impl<S> GenericAuthorStyles<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Create an empty AuthorStyles.
    #[inline]
    pub fn new() -> Self {
        Self {
            stylesheets: AuthorStylesheetSet::new(),
            data: EMPTY_CASCADE_DATA.clone(),
        }
    }

    /// Flush the pending sheet changes, updating `data` as appropriate.
    ///
    /// TODO(emilio): Need a host element and a snapshot map to do invalidation
    /// properly.
    #[inline]
    pub fn flush<E>(&mut self, stylist: &mut Stylist, guard: &SharedRwLockReadGuard)
    where
        E: TElement,
        S: ToMediaListKey,
    {
        let flusher = self
            .stylesheets
            .flush::<E>(/* host = */ None, /* snapshot_map = */ None);

        let result = stylist.rebuild_author_data(&self.data, flusher.sheets, guard);
        if let Ok(Some(new_data)) = result {
            self.data = new_data;
        }
    }
}
