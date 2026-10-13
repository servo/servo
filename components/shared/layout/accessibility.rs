/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::LazyLock;

use accesskit::Role;
use bitflags::bitflags;
use rustc_hash::{FxHashMap, FxHashSet};
use style::Atom;

bitflags! {
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    pub struct AccessibilityDamage: u16 {
        /// The properties of this node (other than children) have changed.
        const Node = 0b0001;
        /// Children have been added to or removed from this node.
        const Children = 0b0010;
        /// This node and its descendants have bounds damage from layout.
        const Layout = 0b0100;

        /// All properties of this node need to be recomputed.
        const Rebuild = 0b0111;
    }
}
malloc_size_of::malloc_size_of_is_0!(AccessibilityDamage);

/// A safe-list of ARIA role attribute values which we map to AccessKit roles.
///
/// This is currently just the roles that don't have any [supported] or [required] properties
/// and also don't require an [accessible name].
///
/// [supported]: https://w3c.github.io/aria/#supportedState
/// [required]: https://w3c.github.io/aria/#requiredState
/// [accessible name]: https://w3c.github.io/aria/#namefromauthor
pub static SUPPORTED_ARIA_ROLES: LazyLock<FxHashSet<Atom>> = LazyLock::new(|| {
    [
        Atom::from("alert"),
        Atom::from("article"),
        Atom::from("banner"),
        Atom::from("blockquote"),
        Atom::from("caption"),
        Atom::from("code"),
        Atom::from("complementary"),
        Atom::from("contentinfo"),
        Atom::from("definition"),
        Atom::from("deletion"),
        Atom::from("directory"),
        Atom::from("document"),
        Atom::from("emphasis"),
        Atom::from("feed"),
        Atom::from("figure"),
        Atom::from("generic"),
        Atom::from("heading"),
        Atom::from("insertion"),
        Atom::from("link"),
        Atom::from("list"),
        Atom::from("log"),
        Atom::from("main"),
        Atom::from("math"),
        Atom::from("navigation"),
        Atom::from("none"),
        Atom::from("note"),
        Atom::from("paragraph"),
        Atom::from("presentation"),
        Atom::from("rowgroup"),
        Atom::from("search"),
        Atom::from("status"),
        Atom::from("strong"),
        Atom::from("term"),
        Atom::from("time"),
        Atom::from("timer"),
    ]
    .into_iter()
    .collect()
});

/// A map from ARIA role name to the corresponding [`Role`] in AccessKit.
pub static ARIA_ROLE_TO_ACCESSKIT_ROLE: LazyLock<FxHashMap<Atom, Role>> = LazyLock::new(|| {
    [
        (Atom::from("alert"), Role::Alert),
        (Atom::from("article"), Role::Article),
        (Atom::from("banner"), Role::Banner),
        (Atom::from("blockquote"), Role::Blockquote),
        (Atom::from("caption"), Role::Caption),
        (Atom::from("code"), Role::Code),
        (Atom::from("complementary"), Role::Complementary),
        (Atom::from("contentinfo"), Role::ContentInfo),
        (Atom::from("definition"), Role::Definition),
        (Atom::from("deletion"), Role::ContentDeletion),
        (Atom::from("directory"), Role::Unknown),
        (Atom::from("document"), Role::Document),
        (Atom::from("emphasis"), Role::Emphasis),
        (Atom::from("feed"), Role::Feed),
        (Atom::from("figure"), Role::Figure),
        (Atom::from("generic"), Role::Section),
        (Atom::from("heading"), Role::Heading),
        (Atom::from("insertion"), Role::ContentInsertion),
        (Atom::from("link"), Role::Link),
        (Atom::from("list"), Role::List),
        (Atom::from("log"), Role::Log),
        (Atom::from("main"), Role::Main),
        (Atom::from("math"), Role::Math),
        (Atom::from("navigation"), Role::Navigation),
        (Atom::from("none"), Role::GenericContainer),
        (Atom::from("note"), Role::Note),
        (Atom::from("paragraph"), Role::Paragraph),
        (Atom::from("presentation"), Role::GenericContainer),
        (Atom::from("rowgroup"), Role::RowGroup),
        (Atom::from("search"), Role::Search),
        (Atom::from("separator"), Role::Splitter),
        (Atom::from("status"), Role::Status),
        (Atom::from("strong"), Role::Strong),
        // (Atom::from("subscript"), Role::Subscript), // no corresponding accesskit role.
        // (Atom::from("superscript"), Role::Superscript), // no corresponding accesskit role.
        (Atom::from("term"), Role::Term),
        (Atom::from("time"), Role::Time),
        (Atom::from("timer"), Role::Timer),
    ]
    .into_iter()
    .collect()
});

/// The inverse of the map above, i.e. a map from supported AccessKit role to the corresponding ARIA
/// role.
///
/// This is created by inverting the map above to ensure consistency, then for roles where multiple
/// ARIA roles map to the same AccessKit role an explicit entry is added.
pub static ACCESSKIT_ROLE_TO_ARIA_ROLE: LazyLock<FxHashMap<Role, Atom>> = LazyLock::new(|| {
    let mut accesskit_to_aria: FxHashMap<Role, Atom> = ARIA_ROLE_TO_ACCESSKIT_ROLE
        .iter()
        .map(|(aria_role, accesskit_role)| (*accesskit_role, aria_role.clone()))
        .collect();
    // Note: AccessKit doesn't expose nodes with the `GenericContainer` role by default, so it acts
    // more like ARIA's `none`/`presentation` role.
    accesskit_to_aria.insert(Role::GenericContainer, Atom::from("none"));
    accesskit_to_aria
});
