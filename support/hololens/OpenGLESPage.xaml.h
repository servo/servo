/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "OpenGLES.h"
#include "OpenGLESPage.g.h"
#include "Servo.h"

namespace hlservo {
public
ref class OpenGLESPage sealed {
public:
    OpenGLESPage();
    virtual ~OpenGLESPage();

    internal : OpenGLESPage(OpenGLES* openGLES);

private:
    void OnPageLoaded(Platform::Object ^ sender, Windows::UI::Xaml::RoutedEventArgs ^ e);
    void OnVisibilityChanged(Windows::UI::Core::CoreWindow ^ sender,
        Windows::UI::Core::VisibilityChangedEventArgs ^ args);
    void CreateRenderSurface();
    void DestroyRenderSurface();
    void RecoverFromLostDevice();
    void StartRenderLoop();
    void StopRenderLoop();

    OpenGLES* mOpenGLES;

    EGLSurface mRenderSurface;
    Concurrency::critical_section mRenderSurfaceCriticalSection;
    Windows::Foundation::IAsyncAction ^ mRenderLoopWorker;
    Servo* mServo;
};
}
