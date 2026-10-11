# WebRTC Real-Time Calling Architecture & Implementation Plan

Status: **Active Tracker & Architectural Blueprint**  
Target Platforms: Linux, macOS, Windows, Android  
Target Web Applications: Instagram Web (Direct Video/Audio Calling), Discord Web, Google Meet, WhatsApp Web, Zoom Web

---

## 1. Executive Summary

This document establishes the architecture, component breakdown, and step-by-step implementation plan for enabling **real-time audio and video calling** in Servo. Real-world platforms (Instagram, Discord, Google Meet) require full compliance with the W3C WebRTC 1.0 (Unified Plan) and Media Capture & Streams specifications, backed by a production-ready media pipeline (GStreamer `webrtcbin`).

---

## 2. WebRTC Calling Requirements for Major Platforms

To successfully establish a call on Instagram Web or Google Meet, a browser engine must satisfy four interconnected layers:

```
+---------------------------------------------------------------------------------+
|                                 Web Application                                 |
|          (Instagram Web / Google Meet / Discord Web Calling JavaScript)         |
+---------------------------------------------------------------------------------+
                                         |
                                         v
+---------------------------------------------------------------------------------+
|                              DOM & JavaScript Layer                             |
|  - navigator.mediaDevices.getUserMedia({ audio: true, video: true })            |
|  - HTMLMediaElement.srcObject = mediaStream                                     |
|  - RTCPeerConnection (Unified Plan, Transceivers, SDP Offer/Answer, Trickle ICE)|
|  - MediaStream / MediaStreamTrack (audio/video track state & mute events)       |
+---------------------------------------------------------------------------------+
                                         |
                                         v
+---------------------------------------------------------------------------------+
|                            Permission & Embedder Layer                          |
|  - ServoWebViewDelegate::request_permission(Camera / Microphone)                |
|  - Embedder dialog integration (Allow / Deny prompts)                           |
+---------------------------------------------------------------------------------+
                                         |
                                         v
+---------------------------------------------------------------------------------+
|                       Media Core (components/media/webrtc)                      |
|  - WebRtcController & WebRtcSignaller thread boundaries                         |
|  - DataChannel & MediaStream registration and event dispatch                    |
+---------------------------------------------------------------------------------+
                                         |
                                         v
+---------------------------------------------------------------------------------+
|                     GStreamer Backend (webrtcbin pipeline)                      |
|  - STUN / TURN server resolution and ICE agent (libnice)                        |
|  - DTLS-SRTP handshake & key exchange                                           |
|  - Encoders / Decoders: VP8, VP9, H.264 (video); Opus (audio)                   |
|  - Hardware Capture: autovideosrc/v4l2src/avfvideosrc, autoaudiosrc/pulsesrc   |
|  - Dynamic pad routing to WebRender texture sinks & audio sinks                 |
+---------------------------------------------------------------------------------+
```

---

## 3. Inventory of Existing Foundations in Servo

| Subsystem | Existing State | Missing / Work Needed |
| :--- | :--- | :--- |
| **DOM WebIDL & JS Bindings** | `RTCPeerConnection.webidl`, `RTCDataChannel.webidl`, `MediaDevices.webidl` exist in `components/script_bindings/webidls/`. | `dom_webrtc_enabled` and `dom_webrtc_transceiver_enabled` default to `false` in `components/config/prefs.rs`. Unified Plan transceivers (`addTransceiver`, `getTransceivers`) need completion. |
| **Local Device Capture** | `media_capture.rs` in `components/media/backends/gstreamer/` has `GstMediaDevices::new()` and `create_videoinput_stream` / `create_audioinput_stream`. | `getUserMedia` permission prompt pipeline through `ServoWebViewDelegate` is stubbed; constraints filtering needs dynamic device matching. |
| **Video Playback (`srcObject`)** | `HTMLMediaElement` plays URLs via `playbin`. | `HTMLMediaElement.srcObject` linking to a live `MediaStream` sink needs direct bridge from `webrtcbin` pads to WebRender textures. |
| **GStreamer `webrtcbin`** | `webrtc.rs` in `components/media/backends/gstreamer/` implements `GStreamerWebRtcController` wrapping `gst-plugins-bad::webrtcbin`. | Dynamic stream negotiation, BUNDLE/rtcp-mux SDP parsing, TURN credential handling, and remote pad decoding pipelines need completion. |
| **Embedder C API** | `ServoPermissionType` includes `Camera` and `Microphone`. | Embedder callback `request_permission` is ready in `ffi/capi/webview_delegate.rs` and needs default permission allow-policy when headless. |

---

## 4. Step-by-Step Implementation Roadmap

### Phase 1: DOM Preferences & Unified Plan Modernization
1. **Enable WebRTC Preferences**:
   - Turn on `dom_webrtc_enabled: true` and `dom_webrtc_transceiver_enabled: true` in `components/config/prefs.rs`.
   - Ensure `RTCPeerConnection`, `MediaDevices`, and `MediaStream` are cleanly exposed to `Window` in secure contexts (`https://`).
2. **Unified Plan Transceiver Management**:
   - Complete `RTCRtpTransceiver`, `RTCRtpSender`, and `RTCRtpReceiver` DOM implementations in `components/script/dom/webrtc/`.
   - Update `RTCPeerConnection.addTransceiver()` and `RTCPeerConnection.getTransceivers()`.
   - Implement `ontrack` (`RTCTrackEvent`) dispatching when remote tracks are announced in SDP.

### Phase 2: Local Capture & Permissions Pipeline
1. **Camera & Microphone Permissions**:
   - Connect `navigator.mediaDevices.getUserMedia()` to `WebViewDelegate::request_permission` (`ServoPermissionType::Camera` / `Microphone`).
   - If granted, invoke `ServoMedia::create_audioinput_stream` and `ServoMedia::create_videoinput_stream`.
2. **GStreamer Capture Pipelines**:
   - Verify device monitors (`DeviceMonitor`) across platforms:
     - Linux: `v4l2src` (video), `pulsesrc` / `alsasrc` (audio).
     - macOS: `avfvideosrc` (video), `osxaudiosrc` (audio).
     - Windows: `dshowvideosrc` / `mfvideosrc` (video), `wasapisrc` (audio).
   - Package frames into a local `MediaStreamTrack`.

### Phase 3: GStreamer `webrtcbin` Core Engine
1. **STUN / TURN Configuration**:
   - Parse `RTCConfiguration.iceServers` (URLs, username, credential).
   - Set `stun-server` and `turn-server` properties on `webrtcbin`.
2. **SDP Negotiation**:
   - Complete Offer/Answer generation using `webrtcbin::create-offer` and `webrtcbin::create-answer`.
   - Handle ICE candidate trickle (`webrtcbin::add-ice-candidate` and `on-ice-candidate` signal).
   - Standardize codec payloaders: VP8 (`rtpvp8pay`/`rtpvp8depay`), Opus (`rtpopuspay`/`rtpopusdepay`), and H.264 (`rtph264pay`/`rtph264depay`).

### Phase 4: Remote Track Decoding & Video Element Rendering
1. **`HTMLMediaElement.srcObject`**:
   - When JavaScript sets `video.srcObject = remoteStream`, extract underlying `MediaStreamId`.
2. **Dynamic Sink Linking**:
   - In GStreamer backend, connect `webrtcbin.connect_pad_added`:
     - For audio: link incoming pad to `rtpopusdepay` -> `opusdec` -> `audioconvert` -> `autoaudiosink`.
     - For video: link incoming pad to `rtpvp8depay` / `rtph264depay` -> decoder -> `videoconvert` -> `appsink`.
   - Pass decoded video frames to Servo's video frame queue for WebRender texture presentation.

### Phase 5: Verification & Compatibility Testing
1. **Unit & Integration Tests**:
   - Test loopback call (`RTCPeerConnection` connecting to local `RTCPeerConnection` in an iframe).
   - Verify audio and video frame delivery in `tests/wpt/web-platform-tests/webrtc/`.
2. **Real-World Platform Smoke Testing**:
   - Test on Instagram Web video calling, Google Meet, and Discord Web.

---

## 5. Key Source Files Reference

- **DOM WebRTC**:
  - `components/script/dom/webrtc/rtcpeerconnection.rs`
  - `components/script/dom/webrtc/rtcrtptransceiver.rs`
  - `components/script/dom/webrtc/rtctrackevent.rs`
  - `components/script/dom/media/mediadevices.rs`
  - `components/script/dom/media/mediastream.rs`
- **Media Backend**:
  - `components/media/webrtc/lib.rs`
  - `components/media/backends/gstreamer/webrtc.rs`
  - `components/media/backends/gstreamer/media_capture.rs`
  - `components/media/backends/gstreamer/render/`
- **Embedding API**:
  - `ffi/capi/webview_delegate.rs` (`request_permission`, `ServoPermissionType`)
