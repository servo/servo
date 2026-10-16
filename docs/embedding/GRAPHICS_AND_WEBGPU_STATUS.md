# Graphics, WebGL, and WebGPU Architecture Status

## 1. Graphics Pipeline Overview

Servo's rendering pipeline translates DOM layout display lists into GPU drawing commands using Mozilla's **WebRender**:
- WebRender decomposes scenes into GPU primitives and emits OpenGL / Vulkan / DirectX shaders.
- Hardware rasterization occurs on a dedicated Compositor / Render thread.

---

## 2. WebGL 1.0 & WebGL 2.0 Status

Servo implements WebGL bindings in `components/webgl` on top of standard OpenGL / GLES / ANGLE:

| Platform | WebGL Backend | Status |
| :--- | :--- | :--- |
| **Linux (x86_64 / arm64)** | Mesa EGL / GLX Desktop OpenGL & GLES | Production |
| **Windows (x86_64)** | ANGLE (translates GLES -> Direct3D 11) / WGL | Production |
| **macOS (x86_64 / arm64)** | OpenGL Core Profile / ANGLE Metal | Production |
| **Android** | Android EGL / OpenGLES 3.0 | Production |
| **iOS** | OpenGLES / Metal via ANGLE | Spike / Prototype |

---

## 3. WebGPU Implementation (`wgpu`)

Servo integrates the W3C WebGPU specification using the Rust `wgpu` ecosystem (`components/webgpu`):
1. **Naga Shader Translation:** WGSL (WebGPU Shading Language) shaders are parsed and validated by Naga, translating directly to SPIR-V, MSL (Metal), or HLSL (Direct3D 12).
2. **Multi-process Rendering:** In multiprocess mode, WebGPU commands from content processes are serialized and submitted to the GPU compositor thread using shared texture memory handles (`ExternalImageId`).
