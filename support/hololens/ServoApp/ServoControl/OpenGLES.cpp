/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "OpenGLES.h"

using namespace winrt::Windows::UI::Xaml::Controls;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::Foundation::Collections;

OpenGLES::OpenGLES()
    : mEglConfig(nullptr), mEglDisplay(EGL_NO_DISPLAY),
      mEglContext(EGL_NO_CONTEXT) {
  log("OpenGLES::OpenGLES()");
  Initialize();
}

OpenGLES::~OpenGLES() { Cleanup(); }

void OpenGLES::Initialize() {
  const EGLint configAttributes[] = {EGL_RENDERABLE_TYPE,
                                     EGL_OPENGL_ES2_BIT,
                                     EGL_RED_SIZE,
                                     8,
                                     EGL_GREEN_SIZE,
                                     8,
                                     EGL_BLUE_SIZE,
                                     8,
                                     EGL_ALPHA_SIZE,
                                     8,
                                     EGL_DEPTH_SIZE,
                                     24,
                                     EGL_STENCIL_SIZE,
                                     8,
                                     EGL_NONE};

  const EGLint contextAttributes[] = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE};

  // Based on Angle MS template.

  const EGLint defaultDisplayAttributes[] = {
      // These are the default display attributes, used to request ANGLE's D3D11
      // renderer.
      // eglInitialize will only succeed with these attributes if the hardware
      // supports D3D11 Feature Level 10_0+.
      EGL_PLATFORM_ANGLE_TYPE_ANGLE,
      EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,

      // EGL_EXPERIMENTAL_PRESENT_PATH_ANGLE is an optimization that
      // can have large performance benefits on mobile devices.
      EGL_EXPERIMENTAL_PRESENT_PATH_ANGLE,
      EGL_EXPERIMENTAL_PRESENT_PATH_FAST_ANGLE,

      // EGL_PLATFORM_ANGLE_ENABLE_AUTOMATIC_TRIM_ANGLE is an option that
      // enables ANGLE to automatically call
      // the IDXGIDevice3::Trim method on behalf of the application when it gets
      // suspended.
      // Calling IDXGIDevice3::Trim when an application is suspended is a
      // Windows Store application certification
      // requirement.
      EGL_PLATFORM_ANGLE_ENABLE_AUTOMATIC_TRIM_ANGLE,
      EGL_TRUE,
      EGL_NONE,
  };

  const EGLint fl9_3DisplayAttributes[] = {
      // These can be used to request ANGLE's D3D11 renderer, with D3D11 Feature
      // Level 9_3.
      // These attributes are used if the call to eglInitialize fails with the
      // default display attributes.
      EGL_PLATFORM_ANGLE_TYPE_ANGLE,
      EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,
      EGL_PLATFORM_ANGLE_MAX_VERSION_MAJOR_ANGLE,
      9,
      EGL_PLATFORM_ANGLE_MAX_VERSION_MINOR_ANGLE,
      3,
      EGL_EXPERIMENTAL_PRESENT_PATH_ANGLE,
      EGL_EXPERIMENTAL_PRESENT_PATH_FAST_ANGLE,
      EGL_PLATFORM_ANGLE_ENABLE_AUTOMATIC_TRIM_ANGLE,
      EGL_TRUE,
      EGL_NONE,
  };

  const EGLint warpDisplayAttributes[] = {
      // These attributes can be used to request D3D11 WARP.
      // They are used if eglInitialize fails with both the default display
      // attributes and the 9_3 display attributes.
      EGL_PLATFORM_ANGLE_TYPE_ANGLE,
      EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,
      EGL_PLATFORM_ANGLE_DEVICE_TYPE_ANGLE,
      EGL_PLATFORM_ANGLE_DEVICE_TYPE_D3D_WARP_ANGLE,
      EGL_EXPERIMENTAL_PRESENT_PATH_ANGLE,
      EGL_EXPERIMENTAL_PRESENT_PATH_FAST_ANGLE,
      EGL_PLATFORM_ANGLE_ENABLE_AUTOMATIC_TRIM_ANGLE,
      EGL_TRUE,
      EGL_NONE,
  };

  // eglGetPlatformDisplayEXT is an alternative to eglGetDisplay.
  // It allows us to pass in display attributes, used to configure D3D11.
  PFNEGLGETPLATFORMDISPLAYEXTPROC eglGetPlatformDisplayEXT =
      reinterpret_cast<PFNEGLGETPLATFORMDISPLAYEXTPROC>(
          eglGetProcAddress("eglGetPlatformDisplayEXT"));
  if (!eglGetPlatformDisplayEXT) {
    throw winrt::hresult_error(
        E_FAIL, L"Failed to get function eglGetPlatformDisplayEXT");
  }

  //
  // To initialize the display, we make three sets of calls to
  // eglGetPlatformDisplayEXT and eglInitialize, with varying parameters passed
  // to eglGetPlatformDisplayEXT: 1) The first calls uses
  // "defaultDisplayAttributes" as a parameter. This corresponds to D3D11
  // Feature Level 10_0+. 2) If eglInitialize fails for step 1 (e.g. because
  // 10_0+ isn't supported by the default GPU), then we try again
  //    using "fl9_3DisplayAttributes". This corresponds to D3D11 Feature Level
  //    9_3.
  // 3) If eglInitialize fails for step 2 (e.g. because 9_3+ isn't supported by
  // the default GPU), then we try again
  //    using "warpDisplayAttributes".  This corresponds to D3D11 Feature Level
  //    11_0 on WARP, a D3D11 software rasterizer.
  //

  // This tries to initialize EGL to D3D11 Feature Level 10_0+. See above
  // comment for details.
  mEglDisplay = eglGetPlatformDisplayEXT(
      EGL_PLATFORM_ANGLE_ANGLE, EGL_DEFAULT_DISPLAY, defaultDisplayAttributes);
  if (mEglDisplay == EGL_NO_DISPLAY) {
    throw winrt::hresult_error(E_FAIL, L"Failed to get EGL display");
  }

  if (eglInitialize(mEglDisplay, NULL, NULL) == EGL_FALSE) {
    // This tries to initialize EGL to D3D11 Feature Level 9_3, if 10_0+ is
    // unavailable (e.g. on some mobile devices).
    mEglDisplay = eglGetPlatformDisplayEXT(
        EGL_PLATFORM_ANGLE_ANGLE, EGL_DEFAULT_DISPLAY, fl9_3DisplayAttributes);
    if (mEglDisplay == EGL_NO_DISPLAY) {
      throw winrt::hresult_error(E_FAIL, L"Failed to get EGL display");
    }

    if (eglInitialize(mEglDisplay, NULL, NULL) == EGL_FALSE) {
      // This initializes EGL to D3D11 Feature Level 11_0 on WARP, if 9_3+ is
      // unavailable on the default GPU.
      mEglDisplay = eglGetPlatformDisplayEXT(
          EGL_PLATFORM_ANGLE_ANGLE, EGL_DEFAULT_DISPLAY, warpDisplayAttributes);
      if (mEglDisplay == EGL_NO_DISPLAY) {
        throw winrt::hresult_error(E_FAIL, L"Failed to get EGL display");
      }

      if (eglInitialize(mEglDisplay, NULL, NULL) == EGL_FALSE) {
        // If all of the calls to eglInitialize returned EGL_FALSE then an error
        // has occurred.
        throw winrt::hresult_error(E_FAIL, L"Failed to initialize EGL");
      }
    }
  }

  EGLint numConfigs = 0;
  if ((eglChooseConfig(mEglDisplay, configAttributes, &mEglConfig, 1,
                       &numConfigs) == EGL_FALSE) ||
      (numConfigs == 0)) {
    throw winrt::hresult_error(E_FAIL, L"Failed to choose first EGLConfig");
  }

  mEglContext = eglCreateContext(mEglDisplay, mEglConfig, EGL_NO_CONTEXT,
                                 contextAttributes);
  if (mEglContext == EGL_NO_CONTEXT) {
    throw winrt::hresult_error(E_FAIL, L"Failed to create EGL context");
  }
}

void OpenGLES::Cleanup() {
  if (mEglDisplay != EGL_NO_DISPLAY && mEglContext != EGL_NO_CONTEXT) {
    eglDestroyContext(mEglDisplay, mEglContext);
    mEglContext = EGL_NO_CONTEXT;
  }

  if (mEglDisplay != EGL_NO_DISPLAY) {
    eglTerminate(mEglDisplay);
    mEglDisplay = EGL_NO_DISPLAY;
  }
}

void OpenGLES::Reset() {
  Cleanup();
  Initialize();
}
