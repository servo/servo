#include "pch.h"
#include "App.xaml.h"

using namespace hlservo;

App::App()
{
    InitializeComponent();
}

void App::OnLaunched(Windows::ApplicationModel::Activation::LaunchActivatedEventArgs ^ e)
{
#if _DEBUG
    if (IsDebuggerPresent()) {
        DebugSettings->EnableFrameRateCounter = true;
    }
#endif

    if (mPage == nullptr) {
        mPage = ref new OpenGLESPage(&mOpenGLES);
    }

    Windows::UI::Xaml::Window::Current->Content = mPage;
    Windows::UI::Xaml::Window::Current->Activate();
}
