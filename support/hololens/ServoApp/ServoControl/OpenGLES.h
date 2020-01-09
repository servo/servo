/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

class OpenGLES {
public:
  OpenGLES();
  ~OpenGLES();

  EGLNativeWindowType
  GetNativeWindow(winrt::Windows::UI::Xaml::Controls::SwapChainPanel const &,
                  float dpi);

  void Reset();

private:
  void Initialize();
  void Cleanup();

private:
  EGLDisplay mEglDisplay = EGL_NO_DISPLAY;
  EGLContext mEglContext = nullptr;
  EGLConfig mEglConfig = nullptr;
};
