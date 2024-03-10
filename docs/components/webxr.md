# How WebXR works in Servo

## Terminology

Servo's WebXR implementation involves three main components:
1. The script thread (runs all JS for a page)
2. The WebGL thread (maintains WebGL canvas data and invokes GL operations corresponding to [WebGL APIs](https://registry.khronos.org/webgl/specs/latest/1.0/))
3. The compositor (AKA the main thread)

Additionally, there are a number of WebXR-specific concepts:
* The [discovery object](https://doc.servo.org/webxr_api/trait.DiscoveryAPI.html) (ie. how Servo discovers if a device can provide a WebXR session)
* The [WebXR registry](https://doc.servo.org/webxr_api/struct.MainThreadRegistry.html) (the compositor's interface to WebXR)
* The [layer manager](https://doc.servo.org/webxr_api/layer/trait.LayerManagerAPI.html) (manages WebXR layers for a given session and frame operations on those layers)
* The [layer grand manager](https://doc.servo.org/webxr_api/layer/trait.LayerGrandManagerAPI.html) (manages all layer managers for WebXR sessions)

Finally, there are graphics-specific concepts that are important for the low-level details of rendering with WebXR:
* [surfman](https://github.com/servo/webxr/blob/main/webxr/glwindow/mod.rs#L448-L452) is a crate that abstracts away platform-specific details of OpenGL hardware-accelerated rendering
* [surface](https://doc.servo.org/surfman/platform/unix/default/surface/type.Surface.html) is a hardware buffer that are tied to a specific OpenGL context
* [surface texture](https://doc.servo.org/surfman/platform/unix/default/surface/type.SurfaceTexture.html) is an OpenGL texture that wraps a surface. Surface textures can be shared between OpenGL contexts.
* [surfman context](https://doc.servo.org/surfman/platform/unix/default/context/type.Context.html) represents a particular OpenGL context, and is backed by platform-specific implementations (such as EGL on Unix-based platforms)
* [ANGLE](https://github.com/servo/mozangle/) is an OpenGL implementation on top of Direct3D which is used in Servo to provide a consistent OpenGL backend on Windows-based platforms

## How Servo's compositor starts

The embedder is responsible for creating a window and [triggering the rendering context creation](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/ports/servoshell/headed_window.rs#L134) appropriately.
Servo creates the rendering context by [creating a surfman context](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/gfx/rendering_context.rs#L48-L58) which will
be [used by the compositor](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/servo/lib.rs#L467) for all [web content rendering operations](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/servo/lib.rs#L269-L281).

## How a session starts

When a webpage invokes `navigator.xr.requestSession(..)` through JS, this corresponds to the [XrSystem::RequestSession](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/script/dom/xrsystem.rs#L158) method in Servo.
This method [sends a message](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr-api/registry.rs#L103-L108) to the [WebXR message handler](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr-api/registry.rs#L193-L195)
that lives on the main thread, under the [control of the compositor](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/compositing/compositor.rs#L2049).

The WebXR message handler iterates over all known discovery objects and attempts to [request a session](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr-api/registry.rs#L217-L231)
from each of them. The discovery objects encapsulate creating a session for each supported backend.

As of Feb 3, 2024, there are five WebXR backends:
* [magicleap](https://github.com/servo/webxr/tree/main/webxr/magicleap) - supports Magic Leap 1.0 devices
* [googlevr](https://github.com/servo/webxr/tree/main/webxr/googlevr) - supports Google VR
* [headless](https://github.com/servo/webxr/tree/main/webxr/headless) - supports a window-less, device-less device for automated tests
* [glwindow](https://github.com/servo/webxr/tree/main/webxr/glwindow) - supports a GL-based window for manual testing in desktop environments without real devices
* [openxr](https://github.com/servo/webxr/tree/main/webxr/openxr) - supports devices that implement the OpenXR standard

WebXR sessions need to [create a layer manager](https://github.com/servo/webxr/blob/main/webxr/glwindow/mod.rs#L448-L452)
at some point in order to be able to create and render to WebXR layers. This happens in several steps:
1. some initialization happens on the main thread
2. the main thread [sends a synchronous message](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/canvas/webgl_thread.rs#L3182-L3187) to the WebGL thread
3. the WebGL thread [receives the message](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/canvas/webgl_thread.rs#L392-L396)
4. some backend-specific, graphics-specific initialization happens on the WebGL thread, hidden behind the [layer manager factory](https://doc.servo.org/webxr_api/struct.LayerManagerFactory.html) abstraction
5. the new layer manager is [stored in the WebGL thread](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/canvas/webgl_thread.rs#L3058-L3061)
6. the main thread [receives a unique identifier](https://github.com/servo/servo/blob/d7d0451424faf1bf9c705068bea1aa8cf582d6ad/components/canvas/webgl_thread.rs#L3189-L3196) representing the new layer manager

This cross-thread dance is important because the device performing the rendering often has strict requirements for the compatibility of any
WebGL context that is used for rendering, and most GL state is only observable on the thread that created it.

## How an OpenXR session is created

The OpenXR discovery process starts at [OpenXrDiscovery::request_session](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L309).
The discovery object only has access to whatever state was passed in its constructor, as well as a [SessionBuilder](https://doc.servo.org/webxr_api/struct.SessionBuilder.html)
object that contains values required to create a new session.

Creating an OpenXR session first [creates an OpenXR instance](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L192),
which allows coniguring which extensions are in use. There are different extensions used to initialize OpenXR on different platforms; for Window
the [D3D extension](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L213) can be used
since Servo relies on ANGLE for its OpenGL implementation.

Once an OpenXR instance exists, the session builder is used to create a new WebXR session that [runs in its own thread](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L331).
All WebXR sessions can either [run in a thread](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr-api/session.rs#L513-L538)
or have Servo [run them on the main thread](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr-api/session.rs#L540-L552).
This choice has implications for how the graphics for the WebXR session can be set up, based on what GL state must be available for sharing.

OpenXR's new session thread [initializes an OpenXR device](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L332-L337),
which is responsible for creating the actual OpenXR session. This session object is [created on the WebGL thread](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L864-L878)
as part of creating the OpenXR layer manager, since it relies on [sharing the underlying GPU device](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L443-L460) that the WebGL thread uses.

Once the session object has been created, the main thread can [obtain a copy](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L878)
and resume initializing the [remaining properties](https://github.com/servo/webxr/blob/614420b9830615376563fc6e1d98c52119f97123/webxr/openxr/mod.rs#L882-L1026) of the new device.
