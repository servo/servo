/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#interface-definition

[Constructor(optional RTCConfiguration configuration),
 Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCPeerConnection : EventTarget {
    Promise<RTCSessionDescriptionInit> createOffer(optional RTCOfferOptions options);
    // Promise<RTCSessionDescriptionInit> createAnswer(optional RTCAnswerOptions options);
    // Promise<void>                      setLocalDescription(RTCSessionDescriptionInit description);
    // readonly attribute RTCSessionDescription? localDescription;
    // readonly attribute RTCSessionDescription? currentLocalDescription;
    // readonly attribute RTCSessionDescription? pendingLocalDescription;
    // Promise<void>                      setRemoteDescription(RTCSessionDescriptionInit description);
    // readonly attribute RTCSessionDescription? remoteDescription;
    // readonly attribute RTCSessionDescription? currentRemoteDescription;
    // readonly attribute RTCSessionDescription? pendingRemoteDescription;
    Promise<void>                      addIceCandidate(optional RTCIceCandidateInit candidate);
    // readonly attribute RTCSignalingState      signalingState;
    // readonly attribute RTCIceGatheringState   iceGatheringState;
    // readonly attribute RTCIceConnectionState  iceConnectionState;
    // readonly attribute RTCPeerConnectionState connectionState;
    // readonly attribute boolean?               canTrickleIceCandidates;
    // static sequence<RTCIceServer>      getDefaultIceServers();
    // RTCConfiguration                   getConfiguration();
    // void                               setConfiguration(RTCConfiguration configuration);
    // void                               close();
             attribute EventHandler           onnegotiationneeded;
             attribute EventHandler           onicecandidate;
    //          attribute EventHandler           onicecandidateerror;
    //          attribute EventHandler           onsignalingstatechange;
    //          attribute EventHandler           oniceconnectionstatechange;
    //          attribute EventHandler           onicegatheringstatechange;
    //          attribute EventHandler           onconnectionstatechange;
};

dictionary RTCConfiguration {
    sequence<RTCIceServer>   iceServers;
    RTCIceTransportPolicy    iceTransportPolicy = "all";
    RTCBundlePolicy          bundlePolicy = "balanced";
    RTCRtcpMuxPolicy         rtcpMuxPolicy = "require";
    DOMString                peerIdentity;
    // sequence<RTCCertificate> certificates;
    [EnforceRange]
    octet                    iceCandidatePoolSize = 0;
};

enum RTCIceTransportPolicy {
    "relay",
    "all"
};

enum RTCBundlePolicy {
    "balanced",
    "max-compat",
    "max-bundle"
};

enum RTCRtcpMuxPolicy {
    // At risk due to lack of implementers' interest.
    "negotiate",
    "require"
};

dictionary RTCIceServer {
    required (DOMString or sequence<DOMString>) urls;
             DOMString                          username;
             DOMString /*(DOMString or RTCOAuthCredential)*/  credential;
             RTCIceCredentialType               credentialType = "password";
};

enum RTCIceCredentialType {
    "password",
    "oauth"
};

dictionary RTCOfferAnswerOptions {
    boolean voiceActivityDetection = true;
};

dictionary RTCOfferOptions : RTCOfferAnswerOptions {
    boolean iceRestart = false;
};

dictionary RTCAnswerOptions : RTCOfferAnswerOptions {
};