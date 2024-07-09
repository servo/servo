/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;
use regex::bytes::Regex;
use script_traits::{HistoryEntryReplacement, MsDuration};
use servo_url::ServoUrl;
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::{DeclarativeRefresh, Document};
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::location::NavigationType;
use crate::dom::node::{document_from_node, window_from_node, BindContext, Node, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::timers::OneshotTimerCallback;

#[dom_struct]
pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct RefreshRedirectDue {
    #[no_trace]
    pub url: ServoUrl,
    #[ignore_malloc_size_of = "non-owning"]
    pub window: DomRoot<Window>,
}
impl RefreshRedirectDue {
    pub fn invoke(self) {
        self.window.Location().navigate(
            self.url.clone(),
            HistoryEntryReplacement::Enabled,
            NavigationType::DeclarativeRefresh,
        );
    }
}

impl HTMLMetaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLMetaElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);
            if name == "referrer" {
                self.apply_referrer();
            }
        // https://html.spec.whatwg.org/multipage/#attr-meta-http-equiv
        } else if !self.HttpEquiv().is_empty() {
            // TODO: Implement additional http-equiv candidates
            if self.HttpEquiv().to_ascii_lowercase().as_str() == "refresh" {
                self.declarative_refresh();
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    fn apply_referrer(&self) {
        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            if let Some(head) = parent.downcast::<HTMLHeadElement>() {
                head.set_document_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps>
    fn declarative_refresh(&self) {
        // 2
        let content = self.Content();
        // 1
        if !content.is_empty() {
            // 3
            self.shared_declarative_refresh_steps(content);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps>
    fn shared_declarative_refresh_steps(&self, content: DOMString) {
        // 1
        let document = document_from_node(self);
        if document.will_declaratively_refresh() {
            return;
        }

        // 2-11
        lazy_static::lazy_static! {
            static ref REFRESH_REGEX: Regex = Regex::new(
                r#"(?x)
                ^
                \s* # 3
                ((?<time>\d+)\.?|\.) # 5-6
                [0-9.]* # 8
                (
                    (;|,| ) # 10.1
                    \s* # 10.2
                    (;|,)? # 10.3
                    \s* # 10.4
                    (
                        (U|u)(R|r)(L|l) # 11.2-11.4
                        \s*=\s* # 11.5-11.7
                        ('(?<url1>.*?)'?|"(?<url2>.*?)"?|(?<url3>[^'"].*)) # 11.8 - 11.10
                        |
                        (?<url4>.*)
                    )?
                )?
                $
            "#,
            )
            .unwrap();
        }
        let mut url_record = document.url();
        let captures = if let Some(captures) = REFRESH_REGEX.captures(content.as_bytes()) {
            captures
        } else {
            return;
        };
        let time = if let Some(time_string) = captures.name("time") {
            u64::from_str(&String::from_utf8_lossy(time_string.as_bytes())).unwrap_or(0)
        } else {
            0
        };
        let captured_url = captures.name("url1").or(captures
            .name("url2")
            .or(captures.name("url3").or(captures.name("url4"))));

        if let Some(url_match) = captured_url {
            url_record = if let Ok(url) = ServoUrl::parse_with_base(
                Some(&url_record),
                &String::from_utf8_lossy(url_match.as_bytes()),
            ) {
                url
            } else {
                return;
            }
        }
        // 12-13
        if document.completely_loaded() {
            // TODO: handle active sandboxing flag
            let window = window_from_node(self);
            window.upcast::<GlobalScope>().schedule_callback(
                OneshotTimerCallback::RefreshRedirectDue(RefreshRedirectDue {
                    window: window.clone(),
                    url: url_record,
                }),
                MsDuration::new(time.saturating_mul(1000)),
            );
            document.set_declarative_refresh(DeclarativeRefresh::CreatedAfterLoad);
        } else {
            document.set_declarative_refresh(DeclarativeRefresh::PendingLoad {
                url: url_record,
                time,
            });
        }
    }
}

impl HTMLMetaElementMethods for HTMLMetaElement {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-httpequiv
    make_getter!(HttpEquiv, "http-equiv");
    // https://html.spec.whatwg.org/multipage/#dom-meta-httpequiv
    make_atomic_setter!(SetHttpEquiv, "http-equiv");
}

impl VirtualMethods for HTMLMetaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected {
            self.process_attributes();
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }

        self.process_referrer_attribute();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_connected {
            self.process_referrer_attribute();
        }
    }
}
