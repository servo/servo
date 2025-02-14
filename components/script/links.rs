/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines shared hyperlink behaviour for `<link>`, `<a>`, `<area>` and `<form>` elements.

use html5ever::{local_name, namespace_url, ns};
use malloc_size_of::malloc_size_of_is_0;
use net_traits::request::Referrer;
use script_traits::{LoadData, LoadOrigin, NavigationHistoryBehavior};
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::bindings::codegen::Bindings::AttrBinding::Attr_Binding::AttrMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::referrer_policy_for_element;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::node::NodeTraits;
use crate::dom::types::Element;
use crate::script_runtime::CanGc;

bitflags::bitflags! {
    /// Describes the different relations that can be specified on elements using the `rel`
    /// attribute.
    ///
    /// Refer to <https://html.spec.whatwg.org/multipage/#linkTypes> for more information.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct LinkRelations: u32 {
        /// <https://html.spec.whatwg.org/multipage/#rel-alternate>
        const ALTERNATE = 1;

        /// <https://html.spec.whatwg.org/multipage/#link-type-author>
        const AUTHOR = 1 << 1;

        /// <https://html.spec.whatwg.org/multipage/#link-type-bookmark>
        const BOOKMARK = 1 << 2;

        /// <https://html.spec.whatwg.org/multipage/#link-type-canonical>
        const CANONICAL = 1 << 3;

        /// <https://html.spec.whatwg.org/multipage/#link-type-dns-prefetch>
        const DNS_PREFETCH = 1 << 4;

        /// <https://html.spec.whatwg.org/multipage/#link-type-expect>
        const EXPECT = 1 << 5;

        /// <https://html.spec.whatwg.org/multipage/#link-type-external>
        const EXTERNAL = 1 << 6;

        /// <https://html.spec.whatwg.org/multipage/#link-type-help>
        const HELP = 1 << 7;

        /// <https://html.spec.whatwg.org/multipage/#rel-icon>
        const ICON = 1 << 8;

        /// <https://html.spec.whatwg.org/multipage/#link-type-license>
        const LICENSE = 1 << 9;

        /// <https://html.spec.whatwg.org/multipage/#link-type-next>
        const NEXT = 1 << 10;

        /// <https://html.spec.whatwg.org/multipage/#link-type-manifest>
        const MANIFEST = 1 << 11;

        /// <https://html.spec.whatwg.org/multipage/#link-type-modulepreload>
        const MODULE_PRELOAD = 1 << 12;

        /// <https://html.spec.whatwg.org/multipage/#link-type-nofollow>
        const NO_FOLLOW = 1 << 13;

        /// <https://html.spec.whatwg.org/multipage/#link-type-noopener>
        const NO_OPENER = 1 << 14;

        /// <https://html.spec.whatwg.org/multipage/#link-type-noreferrer>
        const NO_REFERRER = 1 << 15;

        /// <https://html.spec.whatwg.org/multipage/#link-type-opener>
        const OPENER = 1 << 16;

        /// <https://html.spec.whatwg.org/multipage/#link-type-pingback>
        const PING_BACK = 1 << 17;

        /// <https://html.spec.whatwg.org/multipage/#link-type-preconnect>
        const PRECONNECT = 1 << 18;

        /// <https://html.spec.whatwg.org/multipage/#link-type-prefetch>
        const PREFETCH = 1 << 19;

        /// <https://html.spec.whatwg.org/multipage/#link-type-preload>
        const PRELOAD = 1 << 20;

        /// <https://html.spec.whatwg.org/multipage/#link-type-prev>
        const PREV = 1 << 21;

        /// <https://html.spec.whatwg.org/multipage/#link-type-privacy-policy>
        const PRIVACY_POLICY = 1 << 22;

        /// <https://html.spec.whatwg.org/multipage/#link-type-search>
        const SEARCH = 1 << 23;

        /// <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>
        const STYLESHEET = 1 << 24;

        /// <https://html.spec.whatwg.org/multipage/#link-type-tag>
        const TAG = 1 << 25;

        /// <https://html.spec.whatwg.org/multipage/#link-type-terms-of-service>
        const TERMS_OF_SERVICE = 1 << 26;
    }
}

impl LinkRelations {
    /// The set of allowed relations for [`<link>`] elements
    ///
    /// [`<link>`]: https://html.spec.whatwg.org/multipage/#htmllinkelement
    pub(crate) const ALLOWED_LINK_RELATIONS: Self = Self::ALTERNATE
        .union(Self::CANONICAL)
        .union(Self::AUTHOR)
        .union(Self::DNS_PREFETCH)
        .union(Self::EXPECT)
        .union(Self::HELP)
        .union(Self::ICON)
        .union(Self::MANIFEST)
        .union(Self::MODULE_PRELOAD)
        .union(Self::LICENSE)
        .union(Self::NEXT)
        .union(Self::PING_BACK)
        .union(Self::PRECONNECT)
        .union(Self::PREFETCH)
        .union(Self::PRELOAD)
        .union(Self::PREV)
        .union(Self::PRIVACY_POLICY)
        .union(Self::SEARCH)
        .union(Self::STYLESHEET)
        .union(Self::TERMS_OF_SERVICE);

    /// The set of allowed relations for [`<a>`] and [`<area>`] elements
    ///
    /// [`<a>`]: https://html.spec.whatwg.org/multipage/#the-a-element
    /// [`<area>`]: https://html.spec.whatwg.org/multipage/#the-area-element
    pub(crate) const ALLOWED_ANCHOR_OR_AREA_RELATIONS: Self = Self::ALTERNATE
        .union(Self::AUTHOR)
        .union(Self::BOOKMARK)
        .union(Self::EXTERNAL)
        .union(Self::HELP)
        .union(Self::LICENSE)
        .union(Self::NEXT)
        .union(Self::NO_FOLLOW)
        .union(Self::NO_OPENER)
        .union(Self::NO_REFERRER)
        .union(Self::OPENER)
        .union(Self::PREV)
        .union(Self::PRIVACY_POLICY)
        .union(Self::SEARCH)
        .union(Self::TAG)
        .union(Self::TERMS_OF_SERVICE);

    /// The set of allowed relations for [`<form>`] elements
    ///
    /// [`<form>`]: https://html.spec.whatwg.org/multipage/#the-form-element
    pub(crate) const ALLOWED_FORM_RELATIONS: Self = Self::EXTERNAL
        .union(Self::HELP)
        .union(Self::LICENSE)
        .union(Self::NEXT)
        .union(Self::NO_FOLLOW)
        .union(Self::NO_OPENER)
        .union(Self::NO_REFERRER)
        .union(Self::OPENER)
        .union(Self::PREV)
        .union(Self::SEARCH);

    /// Compute the set of relations for an element given its `"rel"` attribute
    ///
    /// This function should only be used with [`<link>`], [`<a>`], [`<area>`] and [`<form>`] elements.
    ///
    /// [`<link>`]: https://html.spec.whatwg.org/multipage/#htmllinkelement
    /// [`<a>`]: https://html.spec.whatwg.org/multipage/#the-a-element
    /// [`<area>`]: https://html.spec.whatwg.org/multipage/#the-area-element
    /// [`<form>`]: https://html.spec.whatwg.org/multipage/#the-form-element
    pub(crate) fn for_element(element: &Element) -> Self {
        let rel = element.get_attribute(&ns!(), &local_name!("rel")).map(|e| {
            let value = e.value();
            (**value).to_owned()
        });

        let mut relations = rel
            .map(|attribute| {
                attribute
                    .split(HTML_SPACE_CHARACTERS)
                    .map(Self::from_single_keyword)
                    .collect()
            })
            .unwrap_or(Self::empty());

        // For historical reasons, "rev=made" is treated as if the "author" relation was specified
        let has_legacy_author_relation = element
            .get_attribute(&ns!(), &local_name!("rev"))
            .is_some_and(|rev| &**rev.value() == "made");
        if has_legacy_author_relation {
            relations |= Self::AUTHOR;
        }

        let allowed_relations = if element.is::<HTMLLinkElement>() {
            Self::ALLOWED_LINK_RELATIONS
        } else if element.is::<HTMLAnchorElement>() || element.is::<HTMLAreaElement>() {
            Self::ALLOWED_ANCHOR_OR_AREA_RELATIONS
        } else if element.is::<HTMLFormElement>() {
            Self::ALLOWED_FORM_RELATIONS
        } else {
            Self::empty()
        };

        relations & allowed_relations
    }

    /// Parse one single link relation keyword
    ///
    /// If the keyword is invalid then `Self::empty()` is returned.
    fn from_single_keyword(keyword: &str) -> Self {
        if keyword.eq_ignore_ascii_case("alternate") {
            Self::ALTERNATE
        } else if keyword.eq_ignore_ascii_case("canonical") {
            Self::CANONICAL
        } else if keyword.eq_ignore_ascii_case("author") {
            Self::AUTHOR
        } else if keyword.eq_ignore_ascii_case("bookmark") {
            Self::BOOKMARK
        } else if keyword.eq_ignore_ascii_case("dns-prefetch") {
            Self::DNS_PREFETCH
        } else if keyword.eq_ignore_ascii_case("expect") {
            Self::EXPECT
        } else if keyword.eq_ignore_ascii_case("external") {
            Self::EXTERNAL
        } else if keyword.eq_ignore_ascii_case("help") {
            Self::HELP
        } else if keyword.eq_ignore_ascii_case("icon") ||
            keyword.eq_ignore_ascii_case("shortcut icon") ||
            keyword.eq_ignore_ascii_case("apple-touch-icon")
        {
            // TODO: "apple-touch-icon" is not in the spec. Where did it come from? Do we need it?
            //       There is also "apple-touch-icon-precomposed" listed in
            //       https://github.com/servo/servo/blob/e43e4778421be8ea30db9d5c553780c042161522/components/script/dom/htmllinkelement.rs#L452-L467
            Self::ICON
        } else if keyword.eq_ignore_ascii_case("manifest") {
            Self::MANIFEST
        } else if keyword.eq_ignore_ascii_case("modulepreload") {
            Self::MODULE_PRELOAD
        } else if keyword.eq_ignore_ascii_case("license") ||
            keyword.eq_ignore_ascii_case("copyright")
        {
            Self::LICENSE
        } else if keyword.eq_ignore_ascii_case("next") {
            Self::NEXT
        } else if keyword.eq_ignore_ascii_case("nofollow") {
            Self::NO_FOLLOW
        } else if keyword.eq_ignore_ascii_case("noopener") {
            Self::NO_OPENER
        } else if keyword.eq_ignore_ascii_case("noreferrer") {
            Self::NO_REFERRER
        } else if keyword.eq_ignore_ascii_case("opener") {
            Self::OPENER
        } else if keyword.eq_ignore_ascii_case("pingback") {
            Self::PING_BACK
        } else if keyword.eq_ignore_ascii_case("preconnect") {
            Self::PRECONNECT
        } else if keyword.eq_ignore_ascii_case("prefetch") {
            Self::PREFETCH
        } else if keyword.eq_ignore_ascii_case("preload") {
            Self::PRELOAD
        } else if keyword.eq_ignore_ascii_case("prev") || keyword.eq_ignore_ascii_case("previous") {
            Self::PREV
        } else if keyword.eq_ignore_ascii_case("privacy-policy") {
            Self::PRIVACY_POLICY
        } else if keyword.eq_ignore_ascii_case("search") {
            Self::SEARCH
        } else if keyword.eq_ignore_ascii_case("stylesheet") {
            Self::STYLESHEET
        } else if keyword.eq_ignore_ascii_case("tag") {
            Self::TAG
        } else if keyword.eq_ignore_ascii_case("terms-of-service") {
            Self::TERMS_OF_SERVICE
        } else {
            Self::empty()
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#get-an-element's-noopener>
    pub(crate) fn get_element_noopener(&self, target_attribute_value: Option<&DOMString>) -> bool {
        // Step 1. If element's link types include the noopener or noreferrer keyword, then return true.
        if self.contains(Self::NO_OPENER) || self.contains(Self::NO_REFERRER) {
            return true;
        }

        // Step 2. If element's link types do not include the opener keyword and
        //         target is an ASCII case-insensitive match for "_blank", then return true.
        let target_is_blank =
            target_attribute_value.is_some_and(|target| target.to_ascii_lowercase() == "_blank");
        if !self.contains(Self::OPENER) && target_is_blank {
            return true;
        }

        // Step 3. Return false.
        false
    }
}

malloc_size_of_is_0!(LinkRelations);

/// <https://html.spec.whatwg.org/multipage/#get-an-element's-target>
pub(crate) fn get_element_target(subject: &Element) -> Option<DOMString> {
    if !(subject.is::<HTMLAreaElement>() ||
        subject.is::<HTMLAnchorElement>() ||
        subject.is::<HTMLFormElement>())
    {
        return None;
    }
    if subject.has_attribute(&local_name!("target")) {
        return Some(subject.get_string_attribute(&local_name!("target")));
    }

    let doc = subject.owner_document().base_element();
    match doc {
        Some(doc) => {
            let element = doc.upcast::<Element>();
            if element.has_attribute(&local_name!("target")) {
                Some(element.get_string_attribute(&local_name!("target")))
            } else {
                None
            }
        },
        None => None,
    }
}

/// <https://html.spec.whatwg.org/multipage/#following-hyperlinks-2>
pub(crate) fn follow_hyperlink(
    subject: &Element,
    relations: LinkRelations,
    hyperlink_suffix: Option<String>,
) {
    // Step 1. If subject cannot navigate, then return.
    if subject.cannot_navigate() {
        return;
    }
    // Step 2, done in Step 7.

    let document = subject.owner_document();
    let window = document.window();

    // Step 3: source browsing context.
    let source = document.browsing_context().unwrap();

    // Step 4-5: target attribute.
    let target_attribute_value =
        if subject.is::<HTMLAreaElement>() || subject.is::<HTMLAnchorElement>() {
            get_element_target(subject)
        } else {
            None
        };

    // Step 6.
    let noopener = relations.get_element_noopener(target_attribute_value.as_ref());

    // Step 7.
    let (maybe_chosen, history_handling) = match target_attribute_value {
        Some(name) => {
            let (maybe_chosen, new) = source.choose_browsing_context(name, noopener);
            let history_handling = if new {
                NavigationHistoryBehavior::Replace
            } else {
                NavigationHistoryBehavior::Push
            };
            (maybe_chosen, history_handling)
        },
        None => (Some(window.window_proxy()), NavigationHistoryBehavior::Push),
    };

    // Step 8.
    let chosen = match maybe_chosen {
        Some(proxy) => proxy,
        None => return,
    };

    if let Some(target_document) = chosen.document() {
        let target_window = target_document.window();
        // Step 9, dis-owning target's opener, if necessary
        // will have been done as part of Step 7 above
        // in choose_browsing_context/create_auxiliary_browsing_context.

        // Step 10, 11. TODO: if parsing the URL failed, navigate to error page.

        let attribute = subject.get_attribute(&ns!(), &local_name!("href")).unwrap();
        let mut href = attribute.Value();

        // Step 11: append a hyperlink suffix.
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=28925
        if let Some(suffix) = hyperlink_suffix {
            href.push_str(&suffix);
        }
        let Ok(url) = document.base_url().join(&href) else {
            return;
        };

        // Step 12.
        let referrer_policy = referrer_policy_for_element(subject);

        // Step 13
        let referrer = if relations.contains(LinkRelations::NO_REFERRER) {
            Referrer::NoReferrer
        } else {
            target_window.as_global_scope().get_referrer()
        };

        // Step 14
        let pipeline_id = target_window.as_global_scope().pipeline_id();
        let secure = target_window.as_global_scope().is_secure_context();
        let load_data = LoadData::new(
            LoadOrigin::Script(document.origin().immutable().clone()),
            url,
            Some(pipeline_id),
            referrer,
            referrer_policy,
            Some(secure),
            Some(document.insecure_requests_policy()),
        );
        let target = Trusted::new(target_window);
        let task = task!(navigate_follow_hyperlink: move || {
            debug!("following hyperlink to {}", load_data.url);
            target.root().load_url(history_handling, false, load_data, CanGc::note());
        });
        target_document
            .owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task);
    };
}
