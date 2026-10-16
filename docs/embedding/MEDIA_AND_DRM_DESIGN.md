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
