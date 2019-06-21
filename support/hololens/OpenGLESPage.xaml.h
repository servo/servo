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
    void OnVisibilityChanged(Windows::UI::Core::CoreWindow ^ sender, Windows::UI::Core::VisibilityChangedEventArgs ^ args);
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
