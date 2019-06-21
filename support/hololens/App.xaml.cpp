/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
