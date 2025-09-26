/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use base::id::WebViewId;
use mime::Mime;
use net_traits::ReferrerPolicy;
use net_traits::mime_classifier::{MediaType, MimeClassifier};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, Destination, Initiator, InsecureRequestsPolicy, Referrer, RequestBuilder,
};
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::fetch::create_a_potential_cors_request;

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

impl LinkProcessingOptions {
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
    pub(crate) fn type_matches_destination(&self) -> bool {
        // Step 1. If type is an empty string, then return true.
        if self.link_type.is_empty() {
            return true;
        }
        // Step 2. If destination is "fetch", then return true.
        //
        // Fetch is handled as an empty string destination in the spec:
        // https://fetch.spec.whatwg.org/#concept-potential-destination-translate
        let Some(destination) = self.destination else {
            return false;
        };
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
    pub(crate) fn preload(self, webview_id: WebViewId) -> Option<RequestBuilder> {
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
