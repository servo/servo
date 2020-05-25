/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::document::{determine_policy_for_token, Document};
use crate::dom::htmlimageelement::{image_fetch_request, FromPictureOrSrcSet};
use crate::dom::htmlscriptelement::script_fetch_request;
use crate::script_module::ScriptFetchOptions;
use crate::stylesheet_loader::stylesheet_fetch_request;
use html5ever::buffer_queue::BufferQueue;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::Tag;
use html5ever::tokenizer::TagKind;
use html5ever::tokenizer::Token;
use html5ever::tokenizer::TokenSink;
use html5ever::tokenizer::TokenSinkResult;
use html5ever::tokenizer::Tokenizer as HtmlTokenizer;
use html5ever::tokenizer::TokenizerResult;
use html5ever::Attribute;
use html5ever::LocalName;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use net_traits::request::CorsSettings;
use net_traits::request::CredentialsMode;
use net_traits::request::ParserMetadata;
use net_traits::request::Referrer;
use net_traits::CoreResourceMsg;
use net_traits::FetchChannels;
use net_traits::IpcSend;
use net_traits::ReferrerPolicy;
use net_traits::ResourceThreads;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;

#[derive(JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<PrefetchSink>,
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for HtmlTokenizer<PrefetchSink> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.sink.trace(trc)
    }
}

impl Tokenizer {
    pub fn new(document: &Document) -> Self {
        let sink = PrefetchSink {
            origin: document.origin().immutable().clone(),
            pipeline_id: document.global().pipeline_id(),
            base_url: None,
            document_url: document.url(),
            referrer: document.global().get_referrer(),
            referrer_policy: document.get_referrer_policy(),
            resource_threads: document.loader().resource_threads().clone(),
            // Initially we set prefetching to false, and only set it
            // true after the first script tag, since that is what will
            // block the main parser.
            prefetching: false,
        };
        let options = Default::default();
        let inner = HtmlTokenizer::new(sink, options);
        Tokenizer { inner }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) {
        while let TokenizerResult::Script(PrefetchHandle) = self.inner.feed(input) {}
    }
}

#[derive(JSTraceable)]
struct PrefetchSink {
    origin: ImmutableOrigin,
    pipeline_id: PipelineId,
    document_url: ServoUrl,
    base_url: Option<ServoUrl>,
    referrer: Referrer,
    referrer_policy: Option<ReferrerPolicy>,
    resource_threads: ResourceThreads,
    prefetching: bool,
}

/// The prefetch tokenizer produces trivial results
struct PrefetchHandle;

impl TokenSink for PrefetchSink {
    type Handle = PrefetchHandle;
    fn process_token(
        &mut self,
        token: Token,
        _line_number: u64,
    ) -> TokenSinkResult<PrefetchHandle> {
        let tag = match token {
            Token::TagToken(ref tag) => tag,
            _ => return TokenSinkResult::Continue,
        };
        match (tag.kind, &tag.name) {
            (TagKind::StartTag, &local_name!("script")) if self.prefetching => {
                if let Some(url) = self.get_url(tag, local_name!("src")) {
                    debug!("Prefetch script {}", url);
                    let cors_setting = self.get_cors_settings(tag, local_name!("crossorigin"));
                    let integrity_metadata = self
                        .get_attr(tag, local_name!("integrity"))
                        .map(|attr| String::from(&attr.value))
                        .unwrap_or_default();
                    let request = script_fetch_request(
                        url,
                        cors_setting,
                        self.origin.clone(),
                        self.pipeline_id,
                        ScriptFetchOptions {
                            referrer: self.referrer.clone(),
                            referrer_policy: self.referrer_policy,
                            integrity_metadata,
                            cryptographic_nonce: String::new(),
                            credentials_mode: CredentialsMode::CredentialsSameOrigin,
                            parser_metadata: ParserMetadata::ParserInserted,
                        },
                    );
                    let _ = self
                        .resource_threads
                        .send(CoreResourceMsg::Fetch(request, FetchChannels::Prefetch));
                }
                TokenSinkResult::RawData(RawKind::ScriptData)
            },
            (TagKind::StartTag, &local_name!("img")) if self.prefetching => {
                if let Some(url) = self.get_url(tag, local_name!("src")) {
                    debug!("Prefetch {} {}", tag.name, url);
                    let request = image_fetch_request(
                        url,
                        self.origin.clone(),
                        self.referrer.clone(),
                        self.pipeline_id,
                        self.get_cors_settings(tag, local_name!("crossorigin")),
                        self.get_referrer_policy(tag, local_name!("referrerpolicy")),
                        FromPictureOrSrcSet::No,
                    );
                    let _ = self
                        .resource_threads
                        .send(CoreResourceMsg::Fetch(request, FetchChannels::Prefetch));
                }
                TokenSinkResult::Continue
            },
            (TagKind::StartTag, &local_name!("link")) if self.prefetching => {
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
                            let request = stylesheet_fetch_request(
                                url,
                                cors_setting,
                                self.origin.clone(),
                                self.pipeline_id,
                                self.referrer.clone(),
                                referrer_policy,
                                integrity_metadata,
                            );
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
                self.prefetching = true;
                TokenSinkResult::Script(PrefetchHandle)
            },
            (TagKind::StartTag, &local_name!("base")) => {
                if let Some(url) = self.get_url(tag, local_name!("href")) {
                    if self.base_url.is_none() {
                        debug!("Setting base {}", url);
                        self.base_url = Some(url);
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
        let base = self.base_url.as_ref().unwrap_or(&self.document_url);
        ServoUrl::parse_with_base(Some(base), &attr.value).ok()
    }

    fn get_referrer_policy(&self, tag: &Tag, name: LocalName) -> Option<ReferrerPolicy> {
        self.get_attr(tag, name)
            .and_then(|attr| determine_policy_for_token(&*attr.value))
            .or(self.referrer_policy)
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
