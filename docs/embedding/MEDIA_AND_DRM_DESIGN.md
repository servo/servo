# Media, WebCodecs, and DRM / Encrypted Media Extensions (EME) Architecture

## 1. Executive Summary

Modern web embedding requires robust video/audio streaming, low-latency media decoding (WebCodecs), and for premium video streaming (Netflix, Spotify, Prime Video), Encrypted Media Extensions (EME) with Content Decryption Modules (CDM).

This document analyzes Servo's media architecture, reviews WebCodecs implementation status, and specifies the integration architecture for proprietary CDMs (Widevine, FairPlay, PlayReady).

---

## 2. Current Media Architecture: GStreamer Backend

Servo delegates HTML5 `<video>` and `<audio>` decoding to GStreamer via the `servo-media` crate:
- **Pipeline Setup:** Uses `playbin` / `playbin3` or custom decodebin pipelines to demux MP4, WebM, and Ogg containers.
- **Video Surface Delivery:** GlTextureSink or shared memory buffers stream decoded video frames into WebRender's image cache (`ExternalImageId`).
- **Platform Gaps:**
  - Linux: Native GStreamer system libraries.
  - Windows & macOS: Requires bundled or dynamically discovered GStreamer runtime libraries. Hardware-accelerated decoding depends on platform plugins (`d3d11` / `vtenc` / `vtdec`).

---

## 3. WebCodecs Status

The W3C WebCodecs specification exposes raw video/audio decoders and encoders directly to JavaScript:
- `VideoDecoder`, `AudioDecoder`, `VideoFrame`, `AudioData`, `EncodedVideoChunk`.
- **Servo Status:** Partial prototype. Requires integrating low-overhead asynchronous packet decoding queues bypassing the full HTMLMediaElement state machine.

---

## 4. DRM and Encrypted Media Extensions (EME)

### 4.1 Legal & Policy Constraints
Open-source browser engines cannot bundle or distribute proprietary Content Decryption Modules (Google Widevine, Apple FairPlay, Microsoft PlayReady) due to licensing, licensing fees, non-disclosure agreements, and cryptographic compliance requirements.

### 4.2 Proposed CDM Host Architecture
To enable commercial embedders to support DRM content without compromising Servo's open-source license:

```
+-----------------------------------------------------------+
|                     Servo Content Process                 |
|                                                           |
|   JavaScript EME API (navigator.requestMediaKeySystemAccess) |
|               |                                           |
|   MediaKeys / MediaKeySession                             |
|               | (Encrypted Samples)                       |
+---------------+-------------------------------------------+
                | IPC
+---------------v-------------------------------------------+
|              External CDM Host Broker Process             |
|                                                           |
|   libwidevinecdm.so / widevinecdm.dll                     |
|     - Decrypt and Decode encrypted video/audio frames     |
|     - Secure hardware pipeline / Protected Media Path     |
+-----------------------------------------------------------+
```

1. **CDM Adapter Interface:** Define a stable C ABI (`servo_cdm_adapter.h`) mirroring the Chromium Content Decryption Module interface.
2. **Out-of-Process CDM Host:** The CDM runs in a dedicated sandboxed helper process to isolate proprietary binary blobs from the main browser engine.
3. **Decrypted Frame Forwarding:** Decrypted video frames are written into secure hardware textures or shared memory for presentation by WebRender.

### 4.3 Native ClearKey CDM (`org.w3.clearkey`)
Under the W3C EME specification, `org.w3.clearkey` is the mandatory baseline key system that does not require proprietary licenses or third-party binary blobs:
- **Keys and Sessions:** Managed via JSON Web Key (JWK) sets per RFC 7517.
- **Decryption:** Implemented using standard AES-128-CTR (Counter Mode) or AES-128-CBC block ciphers directly within the media pipeline.
- **Testing & Verification:** Enables automated WPT conformance tests for `encrypted-media` without external binary dependencies.

---

## 5. WebRTC Architecture & Implementation

Servo's WebRTC subsystem provides real-time peer-to-peer audio, video, and data communication:

```
+-----------------------------------------------------------+
|                     DOM Script Engine                     |
|                                                           |
|   RTCPeerConnection, RTCDataChannel, RTCRtpSender,        |
|   RTCRtpReceiver, RTCRtpTransceiver, RTCIceCandidate      |
|               |                                           |
+---------------+-------------------------------------------+
                |
                v
+-----------------------------------------------------------+
|                 GStreamer webrtcbin Backend               |
|                                                           |
|   - ICE / STUN / TURN gathering via libnice               |
|   - DTLS-SRTP handshake and key exchange                  |
|   - SCTP Data Channels via usrsctp                        |
|   - RTP packetization / depacketization                   |
|   - VP8, VP9, H.264, and Opus codec pipelines             |
+-----------------------------------------------------------+
```

### 5.1 Implemented DOM Components
Servo implements the W3C WebRTC 1.0 specification in `components/script/dom/webrtc/`:
- **`RTCPeerConnection` (`rtcpeerconnection.rs`):** Session description negotiation (`createOffer`, `createAnswer`, `setLocalDescription`, `setRemoteDescription`), ICE connection state tracking, and signaling state machine.
- **`RTCDataChannel` (`rtcdatachannel.rs`):** Bi-directional arbitrary data transfer supporting both string and binary ArrayBuffer/Blob payloads.
- **RTP Transceivers (`rtcrtptransceiver.rs`, `rtcrtpsender.rs`, `rtcrtpreceiver.rs`):** Media stream track management, encoding parameters, and direction control (`sendrecv`, `sendonly`, `recvonly`, `inactive`).
- **ICE Handling (`rtcicecandidate.rs`, `rtcpeerconnectioniceevent.rs`):** Candidate trickling and gathering state transitions.

### 5.2 Hardware Acceleration & Platform Support
Video encoding and decoding in WebRTC pipelines utilize platform-accelerated GStreamer elements:
- **Windows:** Direct3D 11 (`d3d11h264dec`, `d3d11vp8dec`).
- **Linux:** VA-API / VDPAU (`vaapih264dec`, `vaapivp8dec`).
- **macOS:** VideoToolbox (`vtdec`, `vtenc`).

---

## 6. CEF Parity & Production Media Roadmap

1. **ClearKey Native Engine:** Provide built-in AES-128-CTR sample decryption for `org.w3.clearkey`.
2. **CDM Broker Protocol:** Implement the out-of-process CDM host broker enabling embedders to dynamically load `widevinecdm.dll` / `libwidevinecdm.so` when licensed.
3. **WebRTC MediaStream Audio/Video Routing:** Complete microphone/camera capture device enumeration (`navigator.mediaDevices.getUserMedia`) via GStreamer device monitor elements.

