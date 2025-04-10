/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::ops::Deref;

use base::id::{PipelineId, WebViewId};
use content_security_policy::Destination;
use html5ever::buffer_queue::BufferQueue;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{
    Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer as HtmlTokenizer,
};
use html5ever::{Attribute, LocalName, local_name};
use js::jsapi::JSTracer;
use markup5ever::TokenizerResult;
use net_traits::request::{
    CorsSettings, CredentialsMode, InsecureRequestsPolicy, ParserMetadata, Referrer,
};
use net_traits::policy_container::PolicyContainer;
use net_traits::{CoreResourceMsg, FetchChannels, IpcSend, ReferrerPolicy, ResourceThreads};
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::trace::{CustomTraceable, JSTraceable};
use crate::dom::document::{Document, determine_policy_for_token};
use crate::dom::htmlscriptelement::script_fetch_request;
use crate::fetch::create_a_potential_cors_request;
use crate::script_module::ScriptFetchOptions;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: TraceableTokenizer,
}

struct TraceableTokenizer(HtmlTokenizer<PrefetchSink>);

impl Deref for TraceableTokenizer {
    type Target = HtmlTokenizer<PrefetchSink>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for TraceableTokenizer {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        CustomTraceable::trace(&self.0, trc)
    }
}

#[allow(unsafe_code)]
unsafe impl CustomTraceable for PrefetchSink {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        <Self as JSTraceable>::trace(self, trc)
    }
}

impl Tokenizer {
    pub(crate) fn new(document: &Document) -> Self {
        let global = document.global();
        let sink = PrefetchSink {
            origin: document.origin().immutable().clone(),
            pipeline_id: global.pipeline_id(),
            webview_id: document.webview_id(),
            base_url: RefCell::new(None),
            document_url: document.url(),
            referrer: global.get_referrer(),
            referrer_policy: document.get_referrer_policy(),
            resource_threads: document.loader().resource_threads().clone(),
            // Initially we set prefetching to false, and only set it
            // true after the first script tag, since that is what will
            // block the main parser.
            prefetching: Cell::new(false),
            insecure_requests_policy: document.insecure_requests_policy(),
            has_trustworthy_ancestor_origin: document.has_trustworthy_ancestor_or_current_origin(),
            policy_container: global.policy_container(),
        };
        let options = Default::default();
        let inner = TraceableTokenizer(HtmlTokenizer::new(sink, options));
        Tokenizer { inner }
    }

    pub(crate) fn feed(&self, input: &BufferQueue) {
        while let TokenizerResult::Script(PrefetchHandle) = self.inner.feed(input) {}
    }
}

#[derive(JSTraceable)]
struct PrefetchSink {
    #[no_trace]
    origin: ImmutableOrigin,
    #[no_trace]
    pipeline_id: PipelineId,
    #[no_trace]
    webview_id: WebViewId,
    #[no_trace]
    document_url: ServoUrl,
    #[no_trace]
    base_url: RefCell<Option<ServoUrl>>,
    #[no_trace]
    referrer: Referrer,
    #[no_trace]
    referrer_policy: ReferrerPolicy,
    #[no_trace]
    resource_threads: ResourceThreads,
    prefetching: Cell<bool>,
    #[no_trace]
    insecure_requests_policy: InsecureRequestsPolicy,
    has_trustworthy_ancestor_origin: bool,
    #[no_trace]
    policy_container: PolicyContainer,
}

/// The prefetch tokenizer produces trivial results
#[derive(Clone, Copy, JSTraceable)]
struct PrefetchHandle;

impl TokenSink for PrefetchSink {
    type Handle = PrefetchHandle;
    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<PrefetchHandle> {
        let tag = match token {
            Token::TagToken(ref tag) => tag,
            _ => return TokenSinkResult::Continue,
        };
        match (tag.kind, &tag.name) {
            (TagKind::StartTag, &local_name!("script")) if self.prefetching.get() => {
                if let Some(url) = self.get_url(tag, local_name!("src")) {
                    debug!("Prefetch script {}", url);
                    let cors_setting = self.get_cors_settings(tag, local_name!("crossorigin"));
                    let integrity_metadata = self
                        .get_attr(tag, local_name!("integrity"))
                        .map(|attr| String::from(&attr.value))
                        .unwrap_or_default();
                    let cryptographic_nonce = self
                        .get_attr(tag, local_name!("nonce"))
                        .map(|attr| String::from(&attr.value))
                        .unwrap_or_default();
                    let request = script_fetch_request(
                        self.webview_id,
                        url,
                        cors_setting,
                        self.origin.clone(),
                        self.pipeline_id,
                        ScriptFetchOptions {
                            referrer: self.referrer.clone(),
                            referrer_policy: self.referrer_policy,
                            integrity_metadata,
                            cryptographic_nonce,
                            credentials_mode: CredentialsMode::CredentialsSameOrigin,
                            parser_metadata: ParserMetadata::ParserInserted,
                        },
                        self.insecure_requests_policy,
                        self.has_trustworthy_ancestor_origin,
                        self.policy_container.clone(),
                    );
                    let _ = self
                        .resource_threads
                        .send(CoreResourceMsg::Fetch(request, FetchChannels::Prefetch));
                }
                TokenSinkResult::RawData(RawKind::ScriptData)
            },
            (TagKind::StartTag, &local_name!("img")) if self.prefetching.get() => {
                if let Some(url) = self.get_url(tag, local_name!("src")) {
                    debug!("Prefetch {} {}", tag.name, url);
                    let request = create_a_potential_cors_request(
                        Some(self.webview_id),
                        url,
                        Destination::Image,
                        self.get_cors_settings(tag, local_name!("crossorigin")),
                        None,
                        self.referrer.clone(),
                        self.insecure_requests_policy,
                        self.has_trustworthy_ancestor_origin,
                        self.policy_container.clone(),
                    )
                    .origin(self.origin.clone())
                    .pipeline_id(Some(self.pipeline_id))
                    .referrer_policy(self.get_referrer_policy(tag, local_name!("referrerpolicy")));

                    let _ = self
                        .resource_threads
                        .send(CoreResourceMsg::Fetch(request, FetchChannels::Prefetch));
                }
                TokenSinkResult::Continue
            },
            (TagKind::StartTag, &local_name!("link")) if self.prefetching.get() => {
                if let Some(rel) = self.get_attr(tag, local_name!("rel")) {
                    if rel.value.eq_ignore_ascii_case("stylesheet") {
                        if let Some(url) = self.get_url(tag, local_name!("href")) {
                            debug!("Prefetch {} {}", tag.name, url);
                            let cors_setting =
                                self.get_cors_settings(tag, local_name!("crossorigin"));
                            let referrer_policy =
                                self.get_referrer_policy(tag, local_name!("referrerpolicy"));
                            let integrity_metadata = self
                                .get_attr(tag, local_name!("integrity"))
                                .map(|attr| String::from(&attr.value))
                                .unwrap_or_default();

                            // https://html.spec.whatwg.org/multipage/#default-fetch-and-process-the-linked-resource
                            let request = create_a_potential_cors_request(
                                Some(self.webview_id),
                                url,
                                Destination::Style,
                                cors_setting,
                                None,
                                self.referrer.clone(),
                                self.insecure_requests_policy,
                                self.has_trustworthy_ancestor_origin,
                                self.policy_container.clone(),
                            )
                            .origin(self.origin.clone())
                            .pipeline_id(Some(self.pipeline_id))
                            .referrer_policy(referrer_policy)
                            .integrity_metadata(integrity_metadata);

                            let _ = self
                                .resource_threads
                                .send(CoreResourceMsg::Fetch(request, FetchChannels::Prefetch));
                        }
                    }
                }
                TokenSinkResult::Continue
            },
            (TagKind::StartTag, &local_name!("script")) => {
                TokenSinkResult::RawData(RawKind::ScriptData)
            },
            (TagKind::EndTag, &local_name!("script")) => {
                // After the first script tag, the main parser is blocked, so it's worth prefetching.
                self.prefetching.set(true);
                TokenSinkResult::Script(PrefetchHandle)
            },
            (TagKind::StartTag, &local_name!("base")) => {
                if let Some(url) = self.get_url(tag, local_name!("href")) {
                    if self.base_url.borrow().is_none() {
                        debug!("Setting base {}", url);
                        *self.base_url.borrow_mut() = Some(url);
                    }
                }
                TokenSinkResult::Continue
            },
            _ => TokenSinkResult::Continue,
        }
    }
}

impl PrefetchSink {
    fn get_attr<'a>(&'a self, tag: &'a Tag, name: LocalName) -> Option<&'a Attribute> {
        tag.attrs.iter().find(|attr| attr.name.local == name)
    }

    fn get_url(&self, tag: &Tag, name: LocalName) -> Option<ServoUrl> {
        let attr = self.get_attr(tag, name)?;
        let base_url = self.base_url.borrow();
        let base = base_url.as_ref().unwrap_or(&self.document_url);
        ServoUrl::parse_with_base(Some(base), &attr.value).ok()
    }

    fn get_referrer_policy(&self, tag: &Tag, name: LocalName) -> ReferrerPolicy {
        self.get_attr(tag, name)
            .map(|attr| determine_policy_for_token(&attr.value))
            .unwrap_or(self.referrer_policy)
    }

    fn get_cors_settings(&self, tag: &Tag, name: LocalName) -> Option<CorsSettings> {
        let crossorigin = self.get_attr(tag, name)?;
        if crossorigin.value.eq_ignore_ascii_case("anonymous") {
            Some(CorsSettings::Anonymous)
        } else if crossorigin.value.eq_ignore_ascii_case("use-credentials") {
            Some(CorsSettings::UseCredentials)
        } else {
            None
        }
    }
}
