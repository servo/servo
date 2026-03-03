/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use base::id::WebViewId;
use cssparser::match_ignore_ascii_case;
use http::header::HeaderMap;
use hyper_serde::Serde;
use mime::Mime;
use net_traits::fetch::headers::get_decode_and_split_header_name;
use net_traits::mime_classifier::{MediaType, MimeClassifier};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, Destination, Initiator, InsecureRequestsPolicy, PreloadId, PreloadKey, Referrer,
    RequestBuilder, RequestClient, RequestId,
};
use net_traits::response::{Response, ResponseBody};
use net_traits::{
    CoreResourceMsg, FetchMetadata, NetworkError, ReferrerPolicy, ResourceFetchTiming,
};
pub use nom_rfc8288::complete::LinkDataOwned as LinkHeader;
use nom_rfc8288::complete::link_lenient as parse_link_header;
use servo_url::{ImmutableOrigin, ServoUrl};
use strum::IntoStaticStr;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::medialist::MediaList;
use crate::dom::node::NodeTraits;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::types::HTMLLinkElement;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{FetchResponseListener, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;

trait ValueForKeyInLinkHeader {
    fn has_key_in_link_header(&self, key: &str) -> bool;
    fn value_for_key_in_link_header(&self, key: &str) -> Option<&str>;
}

impl ValueForKeyInLinkHeader for LinkHeader {
    fn has_key_in_link_header(&self, key: &str) -> bool {
        self.params.iter().any(|p| p.key == key)
    }
    fn value_for_key_in_link_header(&self, key: &str) -> Option<&str> {
        let param = self.params.iter().find(|p| p.key == key)?;
        param.val.as_deref()
    }
}

#[derive(PartialEq)]
pub(crate) enum LinkProcessingPhase {
    Media,
    PreMedia,
}

/// <https://html.spec.whatwg.org/multipage/#link-processing-options>
#[derive(Debug)]
pub(crate) struct LinkProcessingOptions {
    /// <https://html.spec.whatwg.org/multipage/#link-options-href>
    pub(crate) href: String,
    /// <https://html.spec.whatwg.org/multipage/#link-options-destination>
    pub(crate) destination: Destination,
    /// <https://html.spec.whatwg.org/multipage/#link-options-integrity>
    pub(crate) integrity: String,
    /// <https://html.spec.whatwg.org/multipage/#link-options-type>
    pub(crate) link_type: String,
    /// <https://html.spec.whatwg.org/multipage/#link-options-nonce>
    pub(crate) cryptographic_nonce_metadata: String,
    /// <https://html.spec.whatwg.org/multipage/#link-options-crossorigin>
    pub(crate) cross_origin: Option<CorsSettings>,
    /// <https://html.spec.whatwg.org/multipage/#link-options-referrer-policy>
    pub(crate) referrer_policy: ReferrerPolicy,
    /// <https://html.spec.whatwg.org/multipage/#link-options-policy-container>
    pub(crate) policy_container: PolicyContainer,
    /// <https://html.spec.whatwg.org/multipage/#link-options-source-set>
    pub(crate) source_set: Option<()>,
    /// <https://html.spec.whatwg.org/multipage/#link-options-base-url>
    pub(crate) base_url: ServoUrl,
    /// <https://html.spec.whatwg.org/multipage/#link-options-origin>
    pub(crate) origin: ImmutableOrigin,
    pub(crate) insecure_requests_policy: InsecureRequestsPolicy,
    pub(crate) has_trustworthy_ancestor_origin: bool,
    pub(crate) referrer: Referrer,
    // https://html.spec.whatwg.org/multipage/#link-options-environment
    pub(crate) request_client: RequestClient,
    // https://html.spec.whatwg.org/multipage/#link-options-document
    // TODO
    // https://html.spec.whatwg.org/multipage/#link-options-on-document-ready
    // TODO
    // https://html.spec.whatwg.org/multipage/#link-options-fetch-priority
    // TODO
}

impl LinkProcessingOptions {
    /// <https://html.spec.whatwg.org/multipage/#apply-link-options-from-parsed-header-attributes>
    fn apply_link_options_from_parsed_header(
        &mut self,
        link_object: &LinkHeader,
        rel: &str,
    ) -> bool {
        // Step 1. If rel is "preload":
        if rel == "preload" {
            // Step 1.1. If attribs["as"] does not exist, then return false.
            let Some(as_) = link_object.value_for_key_in_link_header("as") else {
                return false;
            };
            // Step 1.2. Let destination be the result of translating attribs["as"].
            let Some(destination) = Self::translate_a_preload_destination(as_) else {
                // Step 1.3. If destination is null, then return false.
                return false;
            };
            // Step 1.4. Set options's destination to destination.
            self.destination = destination;
        }
        // Step 2. If attribs["crossorigin"] exists and is an ASCII case-insensitive match for one of the
        // CORS settings attribute keywords, then set options's crossorigin to the CORS settings attribute
        // state corresponding to that keyword.
        if let Some(cross_origin) = link_object.value_for_key_in_link_header("crossorigin") {
            self.cross_origin = determine_cors_settings_for_token(cross_origin);
        }
        // Step 3. If attribs["integrity"] exists, then set options's integrity to attribs["integrity"].
        if let Some(integrity) = link_object.value_for_key_in_link_header("integrity") {
            self.integrity = integrity.to_owned();
        }
        // Step 4. If attribs["referrerpolicy"] exists and is an ASCII case-insensitive match for
        // some referrer policy, then set options's referrer policy to that referrer policy.
        if let Some(referrer_policy) = link_object.value_for_key_in_link_header("referrerpolicy") {
            self.referrer_policy = ReferrerPolicy::from(referrer_policy);
        }
        // Step 5. If attribs["nonce"] exists, then set options's nonce to attribs["nonce"].
        if let Some(nonce) = link_object.value_for_key_in_link_header("nonce") {
            self.cryptographic_nonce_metadata = nonce.to_owned();
        }
        // Step 6. If attribs["type"] exists, then set options's type to attribs["type"].
        if let Some(link_type) = link_object.value_for_key_in_link_header("type") {
            self.link_type = link_type.to_owned();
        }
        // Step 7. If attribs["fetchpriority"] exists and is an ASCII case-insensitive match
        // for a fetch priority attribute keyword, then set options's fetch priority to that
        // fetch priority attribute keyword.
        // TODO
        // Step 8. Return true.
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#process-a-link-header>
    fn process_link_header(self, rel: &str, document: &Document) {
        if rel == "preload" {
            // https://html.spec.whatwg.org/multipage/#link-type-preload:process-a-link-header
            // The process a link header step for this type of link given a link processing options options
            // is to preload options.
            if !self.type_matches_destination() {
                return;
            }
            self.preload(document.window().webview_id(), None, document);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#translate-a-preload-destination>
    pub(crate) fn translate_a_preload_destination(
        potential_destination: &str,
    ) -> Option<Destination> {
        // Step 2. Return the result of translating destination.
        Some(match potential_destination {
            "fetch" => Destination::None,
            "font" => Destination::Font,
            "image" => Destination::Image,
            "script" => Destination::Script,
            "style" => Destination::Style,
            "track" => Destination::Track,
            // Step 1. If destination is not "fetch", "font", "image",
            // "script", "style", or "track", then return null.
            _ => return None,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-link-request>
    pub(crate) fn create_link_request(self, webview_id: WebViewId) -> Option<RequestBuilder> {
        // Step 1. Assert: options's href is not the empty string.
        assert!(!self.href.is_empty());

        // Step 3. Let url be the result of encoding-parsing a URL given options's href, relative to options's base URL.
        let Ok(url) = ServoUrl::parse_with_base(Some(&self.base_url), &self.href) else {
            // Step 4. If url is failure, then return null.
            return None;
        };

        // Step 5. Let request be the result of creating a potential-CORS request given
        //         url, options's destination, and options's crossorigin.
        // Step 6. Set request's policy container to options's policy container.
        // Step 7. Set request's integrity metadata to options's integrity.
        // Step 8. Set request's cryptographic nonce metadata to options's cryptographic nonce metadata.
        // Step 9. Set request's referrer policy to options's referrer policy.
        // Step 10. Set request's client to options's environment.
        // FIXME: Step 11. Set request's priority to options's fetch priority.
        let builder = create_a_potential_cors_request(
            Some(webview_id),
            url,
            self.destination,
            self.cross_origin,
            None,
            self.referrer,
        )
        .insecure_requests_policy(self.insecure_requests_policy)
        .has_trustworthy_ancestor_origin(self.has_trustworthy_ancestor_origin)
        .policy_container(self.policy_container)
        .client(self.request_client)
        .initiator(Initiator::Link)
        .origin(self.origin)
        .integrity_metadata(self.integrity)
        .cryptographic_nonce_metadata(self.cryptographic_nonce_metadata)
        .referrer_policy(self.referrer_policy);

        // Step 12. Return request.
        Some(builder)
    }

    /// <https://html.spec.whatwg.org/multipage/#match-preload-type>
    pub(crate) fn type_matches_destination(&self) -> bool {
        // Step 1. If type is an empty string, then return true.
        if self.link_type.is_empty() {
            return true;
        }
        // Step 2. If destination is "fetch", then return true.
        //
        // Fetch is handled as an empty string destination in the spec:
        // https://fetch.spec.whatwg.org/#concept-potential-destination-translate
        let destination = self.destination;
        if destination == Destination::None {
            return true;
        }
        // Step 3. Let mimeTypeRecord be the result of parsing type.
        let Ok(mime_type_record) = Mime::from_str(&self.link_type) else {
            // Step 4. If mimeTypeRecord is failure, then return false.
            return false;
        };
        // Step 5. If mimeTypeRecord is not supported by the user agent, then return false.
        //
        // We currently don't check if we actually support the mime type. Only if we can classify
        // it according to the spec.
        let Some(mime_type) = MimeClassifier::get_media_type(&mime_type_record) else {
            return false;
        };
        // Step 6. If any of the following are true:
        if
        // destination is "audio" or "video", and mimeTypeRecord is an audio or video MIME type;
        ((destination == Destination::Audio || destination == Destination::Video) &&
            mime_type == MediaType::AudioVideo)
            // destination is a script-like destination and mimeTypeRecord is a JavaScript MIME type;
            || (destination.is_script_like() && mime_type == MediaType::JavaScript)
            // destination is "image" and mimeTypeRecord is an image MIME type;
            || (destination == Destination::Image && mime_type == MediaType::Image)
            // destination is "font" and mimeTypeRecord is a font MIME type;
            || (destination == Destination::Font && mime_type == MediaType::Font)
            // destination is "json" and mimeTypeRecord is a JSON MIME type;
            || (destination == Destination::Json && mime_type == MediaType::Json)
            // destination is "style" and mimeTypeRecord's essence is text/css; or
            || (destination == Destination::Style && mime_type_record == mime::TEXT_CSS)
            // destination is "track" and mimeTypeRecord's essence is text/vtt,
            || (destination == Destination::Track && mime_type_record.essence_str() == "text/vtt")
        {
            // then return true.
            return true;
        }
        // Step 7. Return false.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#preload>
    pub(crate) fn preload(
        self,
        webview_id: WebViewId,
        link: Option<Trusted<HTMLLinkElement>>,
        document: &Document,
    ) {
        // Step 1. If options's type doesn't match options's destination, then return.
        //
        // Handled by callers, since we need to check the previous destination type
        assert!(self.type_matches_destination());
        // Step 2. If options's destination is "image" and options's source set is not null,
        // then set options's href to the result of selecting an image source from options's source set.
        // TODO
        // Step 3. Let request be the result of creating a link request given options.
        let Some(request) = self.create_link_request(webview_id) else {
            // Step 4. If request is null, then return.
            return;
        };
        let preload_id = PreloadId::default();
        let request = request.preload_id(preload_id.clone());
        // Step 5. Let unsafeEndTime be 0.
        // TODO
        // Step 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
        //
        // This is performed in `CoreResourceManager::fetch`
        // Step 7. Let key be the result of creating a preload key given request.
        let key = PreloadKey::new(&request);
        // Step 8. If options's document is "pending", then set request's initiator type to "early hint".
        // TODO
        // Step 9. Let controller be null.
        // Step 10. Let reportTiming given a Document document be to report timing for controller
        // given document's relevant global object.
        let url = request.url.clone();
        let fetch_context = LinkFetchContext {
            url,
            link,
            document: Trusted::new(document),
            global: Trusted::new(&document.global()),
            type_: LinkFetchContextType::Preload(key.clone()),
            response_body: vec![],
        };
        document.insert_preloaded_resource(key, preload_id);
        // Step 11. Set controller to the result of fetching request, with processResponseConsumeBody
        // set to the following steps given a response response and null, failure, or a byte sequence bodyBytes:
        document.fetch_background(request, fetch_context);
    }
}

pub(crate) fn determine_cors_settings_for_token(token: &str) -> Option<CorsSettings> {
    match_ignore_ascii_case! { token,
        "anonymous" => Some(CorsSettings::Anonymous),
        "use-credentials" => Some(CorsSettings::UseCredentials),
        _ => None,
    }
}

/// <https://html.spec.whatwg.org/multipage/#extract-links-from-headers>
pub(crate) fn extract_links_from_headers(headers: &Option<Serde<HeaderMap>>) -> Vec<LinkHeader> {
    // Step 1. Let links be a new list.
    let mut links = Vec::new();
    let Some(headers) = headers else {
        return links;
    };
    // Step 2. Let rawLinkHeaders be the result of getting, decoding, and splitting `Link` from headers.
    let Some(raw_link_headers) = get_decode_and_split_header_name("Link", headers) else {
        return links;
    };
    // Step 3. For each linkHeader of rawLinkHeaders:
    for link_header in raw_link_headers {
        // Step 3.1. Let linkObject be the result of parsing linkHeader. [WEBLINK]
        let Ok(parsed_link_header) = parse_link_header(&link_header) else {
            continue;
        };
        for link_object in parsed_link_header {
            let Some(link_object) = link_object else {
                // Step 3.2. If linkObject["target_uri"] does not exist, then continue.
                continue;
            };
            // Step 3.3. Append linkObject to links.
            links.push(link_object.to_owned());
        }
    }
    // Step 4. Return links.
    links
}

/// <https://html.spec.whatwg.org/multipage/#process-link-headers>
pub(crate) fn process_link_headers(
    link_headers: &[LinkHeader],
    document: &Document,
    phase: LinkProcessingPhase,
) {
    let global = document.owner_global();
    // Step 1. Let links be the result of extracting links from response's header list.
    //
    // Already performed once when parsing headers by caller
    // Step 2. For each linkObject in links:
    for link_object in link_headers {
        // Step 2.1. Let rel be linkObject["relation_type"].
        let Some(rel) = link_object.value_for_key_in_link_header("rel") else {
            continue;
        };
        // Step 2.2. Let attribs be linkObject["target_attributes"].
        //
        // Not applicable, that's in `link_object.params`
        // Step 2.3. Let expectedPhase be "media" if either "srcset", "imagesrcset",
        // or "media" exist in attribs; otherwise "pre-media".
        let expected_phase = if link_object.has_key_in_link_header("srcset") ||
            link_object.has_key_in_link_header("imagesrcset") ||
            link_object.has_key_in_link_header("media")
        {
            LinkProcessingPhase::Media
        } else {
            LinkProcessingPhase::PreMedia
        };
        // Step 2.4. If expectedPhase is not phase, then continue.
        if expected_phase != phase {
            continue;
        }
        // Step 2.5. If attribs["media"] exists and attribs["media"] does not match the environment, then continue.
        if let Some(media) = link_object.value_for_key_in_link_header("media") {
            if !MediaList::matches_environment(document, media) {
                continue;
            }
        }
        // Step 2.6. Let options be a new link processing options with
        let mut options = LinkProcessingOptions {
            href: link_object.url.clone(),
            destination: Destination::None,
            integrity: String::new(),
            link_type: String::new(),
            cryptographic_nonce_metadata: String::new(),
            cross_origin: None,
            referrer_policy: ReferrerPolicy::EmptyString,
            policy_container: document.policy_container().to_owned(),
            source_set: None,
            origin: document.origin().immutable().to_owned(),
            base_url: document.base_url(),
            insecure_requests_policy: document.insecure_requests_policy(),
            has_trustworthy_ancestor_origin: document.has_trustworthy_ancestor_or_current_origin(),
            request_client: global.request_client(),
            referrer: global.get_referrer(),
        };
        // Step 2.7. Apply link options from parsed header attributes to options given attribs and rel.
        // If that returned false, then return.
        if !options.apply_link_options_from_parsed_header(link_object, rel) {
            return;
        }
        // Step 2.8. If attribs["imagesrcset"] exists and attribs["imagesizes"] exists,
        // then set options's source set to the result of creating a source set given
        // linkObject["target_uri"], attribs["imagesrcset"], attribs["imagesizes"], and null.
        // TODO
        // Step 2.9. Run the process a link header steps for rel given options.
        options.process_link_header(rel, document);
    }
}

#[derive(Clone, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum LinkFetchContextType {
    Prefetch,
    Preload(PreloadKey),
}

impl From<LinkFetchContextType> for InitiatorType {
    fn from(other: LinkFetchContextType) -> Self {
        let name: &'static str = other.into();
        InitiatorType::LocalName(name.to_owned())
    }
}

pub(crate) struct LinkFetchContext {
    /// The `<link>` element (if any) that caused this fetch
    pub(crate) link: Option<Trusted<HTMLLinkElement>>,

    pub(crate) global: Trusted<GlobalScope>,
    pub(crate) document: Trusted<Document>,

    /// The url being prefetched
    pub(crate) url: ServoUrl,

    /// The type of fetching we perform, used when report timings.
    pub(crate) type_: LinkFetchContextType,

    pub(crate) response_body: Vec<u8>,
}

impl FetchResponseListener for LinkFetchContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, mut chunk: Vec<u8>) {
        if matches!(self.type_, LinkFetchContextType::Preload(..)) {
            self.response_body.append(&mut chunk);
        }
    }

    /// Step 7 of <https://html.spec.whatwg.org/multipage/#link-type-prefetch:fetch-and-process-the-linked-resource-2>
    /// and step 3.1 of <https://html.spec.whatwg.org/multipage/#link-type-preload:fetch-and-process-the-linked-resource-2>
    fn process_response_eof(
        mut self,
        cx: &mut js::context::JSContext,
        _: RequestId,
        response_result: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        // Steps for https://html.spec.whatwg.org/multipage/#preload
        if let LinkFetchContextType::Preload(key) = &self.type_ {
            let response = if response_result.is_ok() {
                // Step 11.1. If bodyBytes is a byte sequence, then set response's body to bodyBytes as a body.
                let response = Response::new(self.url.clone(), timing.clone());
                *response.body.lock() = ResponseBody::Done(std::mem::take(&mut self.response_body));
                response
            } else {
                // Step 11.2. Otherwise, set response to a network error.
                Response::network_error(NetworkError::ResourceLoadError("Failed to preload".into()))
            };
            // Step 11.5. If entry's on response available is null, then set entry's response to response;
            // otherwise call entry's on response available given response.
            // Step 12. Let commit be the following steps given a Document document:
            // Step 12.1. If entry's response is not null, then call reportTiming given document.
            // Step 12.2. Set document's map of preloaded resources[key] to entry.
            // Step 13. If options's document is null, then set options's on document ready to commit. Otherwise, call commit with options's document.
            let document = self.document.root();
            let document_preloaded_resources = document.preloaded_resources();
            let Some(preload_id) = document_preloaded_resources.get(key) else {
                unreachable!(
                    "Must only be able to lookup preloaded resources if they already exist in document"
                );
            };
            let _ = self.global.root().core_resource_thread().send(
                CoreResourceMsg::StorePreloadedResponse(preload_id.clone(), response),
            );
        }

        submit_timing(&self, &response_result, &timing, CanGc::from_cx(cx));

        // Step 11.6. If processResponse is given, then call processResponse with response.
        //
        // Part of Preload
        //
        // Step 6. Let processPrefetchResponse be the following steps given a response response and null, failure, or a byte sequence bytesOrNull:
        //
        // Part of Prefetch
        if let Some(link) = self.link.as_ref() {
            link.root()
                .fire_event_after_response(response_result, CanGc::from_cx(cx));
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for LinkFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (self.type_.clone().into(), self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}
