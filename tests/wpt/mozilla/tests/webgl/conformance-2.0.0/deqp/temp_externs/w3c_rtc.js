/*
 * Copyright 2012 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @fileoverview Definitions for components of the WebRTC browser API.
 * @see http://dev.w3.org/2011/webrtc/editor/webrtc.html
 * @see http://tools.ietf.org/html/draft-ietf-rtcweb-jsep-01
 * @see http://www.w3.org/TR/mediacapture-streams/
 *
 * @externs
 * @author bemasc@google.com (Benjamin M. Schwartz)
 */

/**
 * @typedef {string}
 * @see {http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 *     #idl-def-MediaStreamTrackState}
 * In WebIDL this is an enum with values 'live', 'mute', and 'ended',
 * but there is no mechanism in Closure for describing a specialization of
 * the string type.
 */
var MediaStreamTrackState;

/**
 * @interface
 */
function SourceInfo() {}

/** @const {string} */
SourceInfo.prototype.kind;

/** @const {string} */
SourceInfo.prototype.id;

/** @const {?string} */
SourceInfo.prototype.label;

/** @const {boolean} */
SourceInfo.prototype.facing;

/**
 * @interface
 * @see http://www.w3.org/TR/mediacapture-streams/#mediastreamtrack
 */
function MediaStreamTrack() {}

/**
 * @param {!function(!Array.<!SourceInfo>)} callback
 */
MediaStreamTrack.getSources = function(callback) {};

/**
 * @type {string}
 * @const
 */
MediaStreamTrack.prototype.kind;

/**
 * @type {string}
 * @const
 */
MediaStreamTrack.prototype.id;

/**
 * @type {string}
 * @const
 */
MediaStreamTrack.prototype.label;

/**
 * @type {boolean}
 */
MediaStreamTrack.prototype.enabled;

/**
 * @type {MediaStreamTrackState}
 * Read only.
 */
MediaStreamTrack.prototype.readyState;

/**
 * @type {?function(!Event)}
 */
MediaStreamTrack.prototype.onmute;

/**
 * @type {?function(!Event)}
 */
MediaStreamTrack.prototype.onunmute;

/**
 * @type {?function(!Event)}
 */
MediaStreamTrack.prototype.onended;

/**
 * @return {!MediaStreamTrack}
 */
MediaStreamTrack.prototype.clone = function() {};

/** @return {void} */
MediaStreamTrack.prototype.stop = function() {};

/**
 * @constructor
 * @extends {Event}
 * @private
 * @see http://dev.w3.org/2011/webrtc/editor/
 * webrtc-20120720.html#mediastreamtrackevent
 * TODO(bemasc): Update this link to the final definition once one exists
 * (https://www.w3.org/Bugs/Public/show_bug.cgi?id=19568)
 */
function MediaStreamTrackEvent() {}

/**
 * @type {!MediaStreamTrack}
 * @const
 */
MediaStreamTrackEvent.prototype.track;

/**
 * @param {!MediaStream|!Array.<!MediaStreamTrack>=} streamOrTracks
 * @constructor
 * @implements {EventTarget}
 * @see http://www.w3.org/TR/mediacapture-streams/#mediastream
 */
function MediaStream(streamOrTracks) {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
MediaStream.prototype.addEventListener = function(type, listener,
    opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
MediaStream.prototype.removeEventListener = function(type, listener,
    opt_useCapture) {};

/** @override */
MediaStream.prototype.dispatchEvent = function(evt) {};

/**
 * TODO(bemasc): Remove this property.
 * @deprecated
 * @type {string}
 * @const
 */
MediaStream.prototype.label;

/**
 * @type {string}
 * @const
 */
MediaStream.prototype.id;

/**
 * @return {!Array.<!MediaStreamTrack>}
 */
MediaStream.prototype.getAudioTracks = function() {};

/**
 * @return {!Array.<!MediaStreamTrack>}
 */
MediaStream.prototype.getVideoTracks = function() {};

/**
 * @param {string} trackId
 * @return {MediaStreamTrack}
 */
MediaStream.prototype.getTrackById = function(trackId) {};

/**
 * @param {!MediaStreamTrack} track
 */
MediaStream.prototype.addTrack = function(track) {};

/**
 * @param {!MediaStreamTrack} track
 */
MediaStream.prototype.removeTrack = function(track) {};

/**
 * @type {boolean}
 */
MediaStream.prototype.ended;

/**
 * @type {?function(!Event)}
 */
MediaStream.prototype.onended;

/**
 * @type {?function(!MediaStreamTrackEvent)}
 */
MediaStream.prototype.onaddtrack;

/**
 * @type {?function(!MediaStreamTrackEvent)}
 */
MediaStream.prototype.onremovetrack;

/**
 * @deprecated
 * TODO(bemasc): Remove this method once browsers have updated to
 * MediaStreamTrack.stop().
 */
MediaStream.prototype.stop = function() {};

/**
 * @type {function(new: MediaStream,
 *                 (!MediaStream|!Array.<!MediaStreamTrack>)=)}
 */
var webkitMediaStream;

/**
 * This interface defines the available constraint attributes.  These are the
 * attributes defined in
 * {@see http://tools.ietf.org/html/draft-alvestrand-constraints-resolution-01}.
 * Note that although that draft refers to "Media Constraints", the W3C uses
 * the terms "Media[Stream|Track]Constraints" for this type, and
 * defines a different type (for RTCPeerConnection) called "MediaConstraints".
 *
 * This interface type is not part of any standard, so it is marked as private.
 * It is defined here in order to reserve the property names, which would
 * otherwise be rewritten when the compiler processes an object literal.
 * Several subsequent interfaces are defined in the same pattern.
 *
 * Note that although this list includes all the properties supported by
 * libjingle (and hence by Chromium), browsers are permitted to offer other
 * properties as well ({
 * @see http://tools.ietf.org/html/draft-burnett-rtcweb-constraints-registry-02
 * }), and browsers are expected to silently ignore unknown properties.  This
 * creates the potential for a very confusing situation in which properties
 * not listed here are renamed by the compiler and then ignored by the browser.
 *
 * @interface
 * @private
 */
function MediaTrackConstraintSetInterface_() {}

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.minWidth;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.maxWidth;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.minHeight;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.maxHeight;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.minAspectRatio;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.maxAspectRatio;

/**
 * Due to a typo, this is called "minFramerate" in the -01 draft.
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.minFrameRate;

/**
 * @type {?number}
 */
MediaTrackConstraintSetInterface_.prototype.maxFrameRate;

/**
 * This type and two more below are defined as unions with Object because they
 * are normally used as record types by constructing an Object literal, but all
 * of their properties are optional.
 * @typedef {Object|MediaTrackConstraintSetInterface_}
 */
var MediaTrackConstraintSet;

/**
 * @interface
 * @private
 */
function MediaTrackConstraintsInterface_() {}

/**
 * @type {?MediaTrackConstraintSet}
 */
MediaTrackConstraintsInterface_.prototype.mandatory;

/**
 * @type {?Array.<!MediaTrackConstraintSet>}
 */
MediaTrackConstraintsInterface_.prototype.optional;

/**
 * @typedef {Object|MediaTrackConstraintsInterface_}
 */
var MediaTrackConstraints;

/**
 * @interface
 * @private
 */
function MediaStreamConstraintsInterface_() {}

/**
 * @type {boolean|MediaTrackConstraints}
 */
MediaStreamConstraintsInterface_.prototype.audio;

/**
 * @type {boolean|MediaTrackConstraints}
 */
MediaStreamConstraintsInterface_.prototype.video;

/**
 * @typedef {Object|MediaStreamConstraintsInterface_}
 */
var MediaStreamConstraints;

/**
 * @see {http://dev.w3.org/2011/webrtc/editor/getusermedia.html#
 *     navigatorusermediaerror-and-navigatorusermediaerrorcallback}
 * @interface
 */
function NavigatorUserMediaError() {}

/**
 * @type {number}
 * @deprecated Removed from the standard and some browsers.
 * @const
 */
NavigatorUserMediaError.prototype.PERMISSION_DENIED;  /** 1 */

/**
 * @type {number}
 * @deprecated Removed from the standard and some browsers.
 * Read only.
 */
NavigatorUserMediaError.prototype.code;

/**
 * @type {string}
 * Read only.
 */
NavigatorUserMediaError.prototype.name;

/**
 * @type {?string}
 * Read only.
 */
NavigatorUserMediaError.prototype.message;

/**
 * @type {?string}
 * Read only.
 */
NavigatorUserMediaError.prototype.constraintName;

/**
 * @param {MediaStreamConstraints} constraints A MediaStreamConstraints object.
 * @param {function(!MediaStream)} successCallback
 *     A NavigatorUserMediaSuccessCallback function.
 * @param {function(!NavigatorUserMediaError)=} errorCallback A
 *     NavigatorUserMediaErrorCallback function.
 * @see http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 * @see http://www.w3.org/TR/mediacapture-streams/
 */
Navigator.prototype.webkitGetUserMedia =
  function(constraints, successCallback, errorCallback) {};

/**
 * @param {string} type
 * @param {!Object} eventInitDict
 * @constructor
 */
function MediaStreamEvent(type, eventInitDict) {}

/**
 * @type {?MediaStream}
 * @const
 */
MediaStreamEvent.prototype.stream;

/**
 * @typedef {string}
 * @see http://www.w3.org/TR/webrtc/#rtcsdptype
 * In WebIDL this is an enum with values 'offer', 'pranswer', and 'answer',
 * but there is no mechanism in Closure for describing a specialization of
 * the string type.
 */
var RTCSdpType;

/**
 * @param {!Object=} descriptionInitDict The RTCSessionDescriptionInit
 * dictionary.  This optional argument may have type
 * {type:RTCSdpType, sdp:string}, but neither of these keys are required to be
 * present, and other keys are ignored, so the closest Closure type is Object.
 * @constructor
 * @see http://dev.w3.org/2011/webrtc/editor/webrtc.html#rtcsessiondescription-class
 */
function RTCSessionDescription(descriptionInitDict) {}

/**
 * @type {?RTCSdpType}
 * @see http://www.w3.org/TR/webrtc/#widl-RTCSessionDescription-type
 */
RTCSessionDescription.prototype.type;

/**
 * @type {?string}
 * @see http://www.w3.org/TR/webrtc/#widl-RTCSessionDescription-sdp
 */
RTCSessionDescription.prototype.sdp;

/**
 * TODO(bemasc): Remove this definition once it is removed from the browser.
 * @param {string} label The label index (audio/video/data -> 0,1,2)
 * @param {string} sdp The ICE candidate in SDP text form
 * @constructor
 */
function IceCandidate(label, sdp) {}

/**
 * @return {string}
 */
IceCandidate.prototype.toSdp = function() {};

/**
 * @type {?string}
 */
IceCandidate.prototype.label;

/**
 * @param {!Object=} candidateInitDict  The RTCIceCandidateInit dictionary.
 * This optional argument may have type
 * {candidate: string, sdpMid: string, sdpMLineIndex:number}, but none of
 * these keys are required to be present, and other keys are ignored, so the
 * closest Closure type is Object.
 * @constructor
 * @see http://www.w3.org/TR/webrtc/#rtcicecandidate-type
 */
function RTCIceCandidate(candidateInitDict) {}

/**
 * @type {?string}
 */
RTCIceCandidate.prototype.candidate;

/**
 * @type {?string}
 */
RTCIceCandidate.prototype.sdpMid;

/**
 * @type {?number}
 */
RTCIceCandidate.prototype.sdpMLineIndex;

/**
 * @typedef {{url: string}}
 * @private
 * @see http://www.w3.org/TR/webrtc/#rtciceserver-type
 * This dictionary type also has an optional key {credential: ?string}.
 */
var RTCIceServerRecord_;

/**
 * @interface
 * @private
 */
function RTCIceServerInterface_() {}

/**
 * @type {string}
 */
RTCIceServerInterface_.prototype.url;

/**
 * @type {?string}
 */
RTCIceServerInterface_.prototype.credential;

/**
 * This type, and several below it, are constructed as unions between records
 *
 * @typedef {RTCIceServerRecord_|RTCIceServerInterface_}
 * @private
 */
var RTCIceServer;

/**
 * @typedef {{iceServers: !Array.<!RTCIceServer>}}
 * @private
 */
var RTCConfigurationRecord_;

/**
 * @interface
 * @private
 */
function RTCConfigurationInterface_() {}

/**
 * @type {!Array.<!RTCIceServer>}
 */
RTCConfigurationInterface_.prototype.iceServers;

/**
 * @typedef {RTCConfigurationRecord_|RTCConfigurationInterface_}
 */
var RTCConfiguration;

/**
 * @typedef {function(!RTCSessionDescription)}
 */
var RTCSessionDescriptionCallback;

/**
 * @typedef {function(string)}
 */
var RTCPeerConnectionErrorCallback;

/**
 * @typedef {function()}
 */
var RTCVoidCallback;

/**
 * @typedef {string}
 */
var RTCSignalingState;

/**
 * @typedef {string}
 */
var RTCIceConnectionState;

/**
 * @typedef {string}
 */
var RTCIceGatheringState;

/**
 * @param {string} type
 * @param {!Object} eventInitDict
 * @constructor
 */
function RTCPeerConnectionIceEvent(type, eventInitDict) {}

/**
 * @type {RTCIceCandidate}
 * @const
 */
RTCPeerConnectionIceEvent.prototype.candidate;

// Note: The specification of RTCStats types is still under development.
// Declarations here will be updated and removed to follow the development of
// modern browsers, breaking compatibility with older versions as they become
// obsolete.
/**
 * @interface
 */
function RTCStatsReport() {}

/**
 * @type {Date}
 * @const
 */
RTCStatsReport.prototype.timestamp;

/**
 * @return {!Array.<!string>}
 */
RTCStatsReport.prototype.names = function() {};

/**
 * @param {string} name
 * @return {string}
 */
RTCStatsReport.prototype.stat = function(name) {};

/**
 * @deprecated
 * @type {RTCStatsReport}
 * @const
 */
RTCStatsReport.prototype.local;

/**
 * @deprecated
 * @type {RTCStatsReport}
 * @const
 */
RTCStatsReport.prototype.remote;

/**
 * @type {string}
 * @const
 */
RTCStatsReport.prototype.type;

/**
 * @type {string}
 * @const
 */
RTCStatsReport.prototype.id;

/**
 * TODO(bemasc): Remove this type once it is no longer in use.  It has already
 * been removed from the specification.
 * @typedef {RTCStatsReport}
 * @deprecated
 */
var RTCStatsElement;

/**
 * @interface
 */
function RTCStatsResponse() {}

/**
 * @return {!Array.<!RTCStatsReport>}
 */
RTCStatsResponse.prototype.result = function() {};

/**
 * @typedef {function(!RTCStatsResponse, MediaStreamTrack=)}
 */
var RTCStatsCallback;

/**
 * This type is not yet standardized, so the properties here only represent
 * the current capabilities of libjingle (and hence Chromium).
 * TODO(bemasc): Add a link to the relevant standard once MediaConstraint has a
 * standard definition.
 *
 * @interface
 * @private
 */
function MediaConstraintSetInterface_() {}

/**
 * @type {?boolean}
 */
MediaConstraintSetInterface_.prototype.OfferToReceiveAudio;

/**
 * @type {?boolean}
 */
MediaConstraintSetInterface_.prototype.OfferToReceiveVideo;

/**
 * @type {?boolean}
 */
MediaConstraintSetInterface_.prototype.DtlsSrtpKeyAgreement;

/**
 * @type {?boolean}
 */
MediaConstraintSetInterface_.prototype.RtpDataChannels;

/**
 * TODO(bemasc): Make this type public once it is defined in a standard.
 *
 * @typedef {Object|MediaConstraintSetInterface_}
 * @private
 */
var MediaConstraintSet_;

/**
 * @interface
 * @private
 */
function MediaConstraintsInterface_() {}

/**
 * @type {?MediaConstraintSet_}
 */
MediaConstraintsInterface_.prototype.mandatory;

/**
 * @type {?Array.<!MediaConstraintSet_>}
 */
MediaConstraintsInterface_.prototype.optional;

/**
 * This type is used extensively in
 * {@see http://dev.w3.org/2011/webrtc/editor/webrtc.html} but is not yet
 * defined.
 *
 * @typedef {Object|MediaConstraintsInterface_}
 */
var MediaConstraints;

/**
 * @interface
 */
function RTCDataChannel() {}

/**
 * @type {string}
 * @const
 */
RTCDataChannel.prototype.label;

/**
 * @type {boolean}
 * @const
 */
RTCDataChannel.prototype.reliable;

/**
 * An enumerated string type (RTCDataChannelState) with values:
 * "connecting", "open", "closing", and "closed".
 * @type {string}
 * Read only.
 */
RTCDataChannel.prototype.readyState;

/**
 * @type {number}
 * Read only.
 */
RTCDataChannel.prototype.bufferedAmount;

/**
 * @type {?function(!Event)}
 */
RTCDataChannel.prototype.onopen;

/**
 * @type {?function(!Event)}
 */
RTCDataChannel.prototype.onerror;

/**
 * @type {?function(!Event)}
 */
RTCDataChannel.prototype.onclose;

RTCDataChannel.prototype.close = function() {};

/**
 * @type {?function(!MessageEvent.<*>)}
 */
RTCDataChannel.prototype.onmessage;

/**
 * @type {string}
 */
RTCDataChannel.prototype.binaryType;

/**
 * @param {string|!Blob|!ArrayBuffer|!ArrayBufferView} data
 */
RTCDataChannel.prototype.send = function(data) {};

/**
 * @constructor
 * @extends {Event}
 * @private
 */
function RTCDataChannelEvent() {}

/**
 * @type {!RTCDataChannel}
 * Read only.
 */
RTCDataChannelEvent.prototype.channel;

/**
 * @typedef {{reliable: boolean}}
 */
var RTCDataChannelInitRecord_;

/**
 * @interface
 * @private
 */
function RTCDataChannelInitInterface_() {}

/**
 * @type {boolean}
 */
RTCDataChannelInitInterface_.prototype.reliable;

/**
 * @typedef {RTCDataChannelInitInterface_|RTCDataChannelInitRecord_}
 */
var RTCDataChannelInit;

/**
 * @param {RTCConfiguration} configuration
 * @param {!MediaConstraints=} constraints
 * @constructor
 * @implements {EventTarget}
 */
function RTCPeerConnection(configuration, constraints) {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
RTCPeerConnection.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
RTCPeerConnection.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
RTCPeerConnection.prototype.dispatchEvent = function(evt) {};

/**
 * @param {!RTCSessionDescriptionCallback} successCallback
 * @param {!RTCPeerConnectionErrorCallback=} failureCallback
 * @param {!MediaConstraints=} constraints
 */
RTCPeerConnection.prototype.createOffer = function(successCallback,
    failureCallback, constraints) {};

/**
 * @param {RTCSessionDescriptionCallback} successCallback
 * @param {?RTCPeerConnectionErrorCallback=} failureCallback
 * @param {!MediaConstraints=} constraints
 */
RTCPeerConnection.prototype.createAnswer = function(successCallback,
    failureCallback, constraints) {};

/**
 * @param {!RTCSessionDescription} description
 * @param {!RTCVoidCallback=} successCallback
 * @param {!RTCPeerConnectionErrorCallback=} failureCallback
 */
RTCPeerConnection.prototype.setLocalDescription = function(description,
    successCallback, failureCallback) {};

/**
 * @param {!RTCSessionDescription} description
 * @param {!RTCVoidCallback=} successCallback
 * @param {!RTCPeerConnectionErrorCallback=} failureCallback
 */
RTCPeerConnection.prototype.setRemoteDescription = function(description,
    successCallback, failureCallback) {};

/**
 * @type {?RTCSessionDescription}
 * Read only.
 */
RTCPeerConnection.prototype.localDescription;

/**
 * @type {?RTCSessionDescription}
 * Read only.
 */
RTCPeerConnection.prototype.remoteDescription;

/**
 * @type {RTCSignalingState}
 * Read only.
 */
RTCPeerConnection.prototype.signalingState;

/**
 * @param {?RTCConfiguration=} configuration
 * @param {?MediaConstraints=} constraints
 */
RTCPeerConnection.prototype.updateIce = function(configuration, constraints) {};

/**
 * @param {!RTCIceCandidate} candidate
 */
RTCPeerConnection.prototype.addIceCandidate = function(candidate) {};

/**
 * @type {!RTCIceGatheringState}
 * Read only.
 */
RTCPeerConnection.prototype.iceGatheringState;

/**
 * @type {!RTCIceConnectionState}
 * Read only.
 */
RTCPeerConnection.prototype.iceConnectionState;

/**
 * @return {!Array.<!MediaStream>}
 */
RTCPeerConnection.prototype.getLocalStreams = function() {};

/**
 * @return {!Array.<!MediaStream>}
 */
RTCPeerConnection.prototype.getRemoteStreams = function() {};

/**
 * @param {string} streamId
 * @return {MediaStream}
 */
RTCPeerConnection.prototype.getStreamById = function(streamId) {};

/**
 * @param {?string} label
 * @param {RTCDataChannelInit=} dataChannelDict
 * @return {!RTCDataChannel}
 */
RTCPeerConnection.prototype.createDataChannel =
    function(label, dataChannelDict) {};
/**
 * @param {!MediaStream} stream
 * @param {!MediaConstraints=} constraints
 */
RTCPeerConnection.prototype.addStream = function(stream, constraints) {};

/**
 * @param {!MediaStream} stream
 */
RTCPeerConnection.prototype.removeStream = function(stream) {};

// TODO(bemasc): Add identity provider stuff once implementations exist

/**
 * @param {!RTCStatsCallback} successCallback
 * @param {MediaStreamTrack=} selector
 */
RTCPeerConnection.prototype.getStats = function(successCallback, selector) {};

RTCPeerConnection.prototype.close = function() {};

/**
 * @type {?function(!Event)}
 */
RTCPeerConnection.prototype.onnegotiationneeded;

/**
 * @type {?function(!RTCPeerConnectionIceEvent)}
 */
RTCPeerConnection.prototype.onicecandidate;

/**
 * @type {?function(!Event)}
 */
RTCPeerConnection.prototype.onsignalingstatechange;

/**
 * @type {?function(!MediaStreamEvent)}
 */
RTCPeerConnection.prototype.onaddstream;

/**
 * @type {?function(!MediaStreamEvent)}
 */
RTCPeerConnection.prototype.onremovestream;

/**
 * @type {?function(!Event)}
 */
RTCPeerConnection.prototype.oniceconnectionstatechange;

/**
 * @type {?function(!RTCDataChannelEvent)}
 */
RTCPeerConnection.prototype.ondatachannel;

/**
 * @type {function(new: RTCPeerConnection, RTCConfiguration,
 *     !MediaConstraints=)}
 */
var webkitRTCPeerConnection;
