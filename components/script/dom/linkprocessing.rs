/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use base::id::WebViewId;
use http::header::HeaderMap;
use hyper_serde::Serde;
use mime::Mime;
use net_traits::fetch::headers::get_decode_and_split_header_name;
use net_traits::mime_classifier::{MediaType, MimeClassifier};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, Destination, Initiator, InsecureRequestsPolicy, Referrer, RequestBuilder,
    RequestId,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ReferrerPolicy, ResourceFetchTiming,
    ResourceTimingType,
};
pub use nom_rfc8288::complete::LinkDataOwned as LinkHeader;
use nom_rfc8288::complete::link as parse_link_header;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::{
    Document, determine_cors_settings_for_token, determine_policy_for_token,
};
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#link-processing-options>
pub(crate) struct LinkProcessingOptions {
    pub(crate) href: String,
    pub(crate) destination: Option<Destination>,
    pub(crate) integrity: String,
    pub(crate) link_type: String,
    pub(crate) cryptographic_nonce_metadata: String,
    pub(crate) cross_origin: Option<CorsSettings>,
    pub(crate) referrer_policy: ReferrerPolicy,
    pub(crate) policy_container: PolicyContainer,
    pub(crate) source_set: Option<()>,
    pub(crate) base_url: ServoUrl,
    pub(crate) origin: ImmutableOrigin,
    pub(crate) insecure_requests_policy: InsecureRequestsPolicy,
    pub(crate) has_trustworthy_ancestor_origin: bool,
    // Some fields that we don't need yet are missing
}

trait ValueForKeyInLinkHeader {
    fn value_for_key_in_link_header(&self, key: &str) -> Option<&str>;
}

impl ValueForKeyInLinkHeader for LinkHeader {
    fn value_for_key_in_link_header(&self, key: &str) -> Option<&str> {
        let param = self.params.iter().find(|p| p.key == key)?;
        param.val.as_deref()
    }
}

impl LinkProcessingOptions {
    /// <https://html.spec.whatwg.org/multipage/#extract-links-from-headers>
    pub(crate) fn extract_links_from_headers(
        headers: &Option<Serde<HeaderMap>>,
    ) -> Vec<LinkHeader> {
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
            println!("link header {}", link_header);
            // Step 3.1. Let linkObject be the result of parsing linkHeader. [WEBLINK]
            println!("parsed result: {:?}", parse_link_header(&link_header));
            let Ok(parsed_link_header) = parse_link_header(&link_header) else {
                continue;
            };
            println!("parsed link header {:?}", parsed_link_header);
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
    pub(crate) fn process_link_headers(link_headers: Vec<LinkHeader>, document: &Document) {
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
            // TODO
            // Step 2.4. If expectedPhase is not phase, then continue.
            // TODO
            // Step 2.5. If attribs["media"] exists and attribs["media"] does not match the environment, then continue.
            if let Some(media) = link_object.value_for_key_in_link_header("media") {
                if !document.matches_environment(media) {
                    continue;
                }
            }
            // Step 2.6. Let options be a new link processing options with
            let mut options = LinkProcessingOptions {
                href: link_object.url.clone(),
                destination: None,
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
                has_trustworthy_ancestor_origin: document
                    .has_trustworthy_ancestor_or_current_origin(),
            };
            // Step 2.7. Apply link options from parsed header attributes to options given attribs.
            options.apply_link_options_from_parsed_header(&link_object);
            // Step 2.8. If attribs["imagesrcset"] exists and attribs["imagesizes"] exists,
            // then set options's source set to the result of creating a source set given
            // linkObject["target_uri"], attribs["imagesrcset"], attribs["imagesizes"], and null.
            // TODO
            // Step 2.9. Run the process a link header steps for rel given options.
            options.process_link_header(rel, document);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#apply-link-options-from-parsed-header-attributes>
    fn apply_link_options_from_parsed_header(&mut self, link_object: &LinkHeader) {
        // Step 1. If attribs["as"] exists, then set options's destination to the result of translating attribs["as"].
        if let Some(as_) = link_object.value_for_key_in_link_header("as") {
            self.destination = Some(Self::translate_a_preload_destination(as_));
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
            self.referrer_policy = determine_policy_for_token(referrer_policy);
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
    }

    /// <https://html.spec.whatwg.org/multipage/#process-a-link-header>
    fn process_link_header(self, rel: &str, document: &Document) {
        if rel == "preload" {
            // https://html.spec.whatwg.org/multipage/#link-type-preload:process-a-link-header
            // The process a link header step for this type of link given a link processing options options
            // is to preload options.
            let Some(request) = self.preload(document.window().webview_id()) else {
                return;
            };
            let url = request.url.clone();
            let fetch_context = LinkHeaderFetchContext {
                url,
                global: Trusted::new(&document.global()),
                resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
                type_: "preload".to_owned(),
            };
            document.fetch_background(request, fetch_context);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#translate-a-preload-destination>
    pub(crate) fn translate_a_preload_destination(potential_destination: &str) -> Destination {
        match potential_destination {
            "fetch" => Destination::None,
            "font" => Destination::Font,
            "image" => Destination::Image,
            "script" => Destination::Script,
            "track" => Destination::Track,
            _ => Destination::None,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-link-request>
    pub(crate) fn create_link_request(self, webview_id: WebViewId) -> Option<RequestBuilder> {
        // Step 1. Assert: options's href is not the empty string.
        assert!(!self.href.is_empty());

        // Step 2. If options's destination is null, then return null.
        let destination = self.destination?;

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
        // FIXME: Step 10. Set request's client to options's environment.
        // FIXME: Step 11. Set request's priority to options's fetch priority.
        // FIXME: Use correct referrer
        let builder = create_a_potential_cors_request(
            Some(webview_id),
            url,
            destination,
            self.cross_origin,
            None,
            Referrer::NoReferrer,
            self.insecure_requests_policy,
            self.has_trustworthy_ancestor_origin,
            self.policy_container,
        )
        .initiator(Initiator::Link)
        .origin(self.origin)
        .integrity_metadata(self.integrity)
        .cryptographic_nonce_metadata(self.cryptographic_nonce_metadata)
        .referrer_policy(self.referrer_policy);

        // Step 12. Return request.
        Some(builder)
    }

    /// <https://html.spec.whatwg.org/multipage/#match-preload-type>
    fn type_matches_destination(mime_type: &str, destination: Option<Destination>) -> bool {
        // Step 1. If type is an empty string, then return true.
        if mime_type.is_empty() {
            return true;
        }
        // Step 2. If destination is "fetch", then return true.
        //
        // Fetch is handled as an empty string destination in the spec:
        // https://fetch.spec.whatwg.org/#concept-potential-destination-translate
        let Some(destination) = destination else {
            return false;
        };
        if destination == Destination::None {
            return true;
        }
        // Step 3. Let mimeTypeRecord be the result of parsing type.
        let Ok(mime_type_record) = Mime::from_str(mime_type) else {
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
    pub(crate) fn preload(self, webview_id: WebViewId) -> Option<RequestBuilder> {
        // Step 1. If options's type doesn't match options's destination, then return.
        let type_matches_destination: bool =
            Self::type_matches_destination(&self.link_type, self.destination);
        // self.previous_type_matched.set(type_matches_destination);
        if !type_matches_destination {
            return None;
        }
        // Step 2. If options's destination is "image" and options's source set is not null,
        // then set options's href to the result of selecting an image source from options's source set.
        // TODO
        // Step 3. Let request be the result of creating a link request given options.
        let Some(request) = self.create_link_request(webview_id) else {
            // Step 4. If request is null, then return.
            return None;
        };
        // Step 5. Let unsafeEndTime be 0.
        // TODO
        // Step 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
        // TODO
        // Step 7. Let key be the result of creating a preload key given request.
        // TODO
        // Step 8. If options's document is "pending", then set request's initiator type to "early hint".
        // TODO
        // Step 9. Let controller be null.
        // Step 10. Let reportTiming given a Document document be to report timing for controller
        // given document's relevant global object.
        // Step 11. Set controller to the result of fetching request, with processResponseConsumeBody
        // set to the following steps given a response response and null, failure, or a byte sequence bodyBytes:
        Some(request.clone())
    }
}

struct LinkHeaderFetchContext {
    /// The global associated with processing this header
    global: Trusted<GlobalScope>,

    resource_timing: ResourceFetchTiming,

    /// The url being prefetched
    url: ServoUrl,

    /// The type of fetching we perform, used when report timings.
    type_: String,
}

impl FetchResponseListener for LinkHeaderFetchContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        _ = response;
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for LinkHeaderFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName(self.type_.clone()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

impl PreInvoke for LinkHeaderFetchContext {
    fn should_invoke(&self) -> bool {
        // Prefetch requests are never aborted.
        true
    }
}
