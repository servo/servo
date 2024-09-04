use html5ever::{local_name, namespace_url, ns};
use malloc_size_of::malloc_size_of_is_0;
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::types::Element;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct LinkRelations: u32 {
        const ALTERNATE = 1;
        const AUTHOR = 1 << 1;
        const BOOKMARK = 1 << 2;
        const CANONICAL = 1 << 3;
        const DNS_PREFETCH = 1 << 4;
        const EXPECT = 1 << 5;
        const EXTERNAL = 1 << 6;
        const HELP = 1 << 7;
        const ICON = 1 << 8;
        const LICENSE = 1 << 9;
        const NEXT = 1 << 10;
        const MANIFEST = 1 << 11;
        const MODULE_PRELOAD = 1 << 12;
        const NO_FOLLOW = 1 << 13;
        const NO_OPENER = 1 << 14;
        const NO_REFERRER = 1 << 15;
        const OPENER = 1 << 16;
        const PING_BACK = 1 << 17;
        const PRECONNECT = 1 << 18;
        const PREFETCH = 1 << 19;
        const PRELOAD = 1 << 20;
        const PREV = 1 << 21;
        const PRIVACY_POLICY = 1 << 22;
        const SEARCH = 1 << 23;
        const STYLESHEET = 1 << 24;
        const TAG = 1 << 25;
        const TermsOfService = 1 << 26;
    }
}

impl LinkRelations {
    pub fn for_element(element: &Element) -> Self {
        let rel = element.get_attribute(&ns!(), &local_name!("rel")).map(|e| {
            let value = e.value();
            (**value).to_owned()
        });

        // FIXME: for a, area and form elements we need to allow a different
        //        set of attributes
        let mut relations = rel
            .map(|attribute| {
                attribute
                    .split(HTML_SPACE_CHARACTERS)
                    .map(Self::from_single_keyword_for_link_element)
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

        relations
    }

    /// Parse one of the relations allowed for the `<link>` element
    ///
    /// If the keyword is invalid then `Self::empty` is returned.
    fn from_single_keyword_for_link_element(keyword: &str) -> Self {
        if keyword.eq_ignore_ascii_case("alternate") {
            Self::ALTERNATE
        } else if keyword.eq_ignore_ascii_case("canonical") {
            Self::CANONICAL
        } else if keyword.eq_ignore_ascii_case("author") {
            Self::AUTHOR
        } else if keyword.eq_ignore_ascii_case("dns-prefetch") {
            Self::DNS_PREFETCH
        } else if keyword.eq_ignore_ascii_case("expect") {
            Self::EXPECT
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
        } else if keyword.eq_ignore_ascii_case("terms-of-service") {
            Self::TermsOfService
        } else {
            Self::empty()
        }
    }
}

malloc_size_of_is_0!(LinkRelations);
