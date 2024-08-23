/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#interface-definition

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCPeerConnection : EventTarget {
    [Throws] constructor(optional RTCConfiguration configuration = {});
    Promise<RTCSessionDescriptionInit> createOffer(optional RTCOfferOptions options = {});
    Promise<RTCSessionDescriptionInit> createAnswer(optional RTCAnswerOptions options = {});
    Promise<undefined>                      setLocalDescription(RTCSessionDescriptionInit description);
    readonly attribute RTCSessionDescription? localDescription;
    // readonly attribute RTCSessionDescription? currentLocalDescription;
    // readonly attribute RTCSessionDescription? pendingLocalDescription;
    Promise<undefined>                      setRemoteDescription(RTCSessionDescriptionInit description);
    readonly attribute RTCSessionDescription? remoteDescription;
    // readonly attribute RTCSessionDescription? currentRemoteDescription;
    // readonly attribute RTCSessionDescription? pendingRemoteDescription;
    Promise<undefined>                      addIceCandidate(optional RTCIceCandidateInit candidate = {});
    readonly attribute RTCSignalingState      signalingState;
    readonly attribute RTCIceGatheringState   iceGatheringState;
    readonly attribute RTCIceConnectionState  iceConnectionState;
    // readonly attribute RTCPeerConnectionState connectionState;
    // readonly attribute boolean?               canTrickleIceCandidates;
    // static sequence<RTCIceServer>      getDefaultIceServers();
    // RTCConfiguration                   getConfiguration();
    // void                               setConfiguration(RTCConfiguration configuration);
    undefined                               close();
             attribute EventHandler           onnegotiationneeded;
             attribute EventHandler           onicecandidate;
    //          attribute EventHandler           onicecandidateerror;
             attribute EventHandler           onsignalingstatechange;
             attribute EventHandler           oniceconnectionstatechange;
             attribute EventHandler           onicegatheringstatechange;
    //          attribute EventHandler           onconnectionstatechange;

    // removed from spec, but still shipped by browsers
    undefined addStream (MediaStream stream);
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

enum RTCIceGatheringState {
    "new",
    "gathering",
    "complete"
};

enum RTCIceConnectionState {
    "new",
    "checking",
    "connected",
    "completed",
    "disconnected",
    "failed",
    "closed"
};

enum RTCSignalingState {
    "stable",
    "have-local-offer",
    "have-remote-offer",
    "have-local-pranswer",
    "have-remote-pranswer",
    "closed"
};

dictionary RTCRtpCodingParameters {
  DOMString rid;
};

dictionary RTCRtpEncodingParameters : RTCRtpCodingParameters {
  boolean active = true;
  unsigned long maxBitrate;
  double scaleResolutionDownBy;
};

dictionary RTCRtpTransceiverInit {
  RTCRtpTransceiverDirection direction = "sendrecv";
  sequence<MediaStream> streams = [];
  sequence<RTCRtpEncodingParameters> sendEncodings = [];
};

partial interface RTCPeerConnection {
    // sequence<RTCRtpSender>      getSenders();
    // sequence<RTCRtpReceiver>    getReceivers();
    // sequence<RTCRtpTransceiver> getTransceivers();
    // RTCRtpSender                addTrack(MediaStreamTrack track,
    //                                      MediaStream... streams);
    // void                        removeTrack(RTCRtpSender sender);
    [Pref="dom.webrtc.transceiver.enabled"]
    RTCRtpTransceiver           addTransceiver((MediaStreamTrack or DOMString) trackOrKind,
                                               optional RTCRtpTransceiverInit init = {});
    attribute EventHandler ontrack;
};

// https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface-extensions-0
partial interface RTCPeerConnection {
  // readonly attribute RTCSctpTransport? sctp;
  RTCDataChannel createDataChannel(USVString label,
                                   optional RTCDataChannelInit dataChannelDict = {});
  attribute EventHandler ondatachannel;
};
