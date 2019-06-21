/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "OpenGLES.h"
#include "app.g.h"
#include "openglespage.xaml.h"

namespace hlservo {
ref class App sealed {
public:
    App();
    virtual void OnLaunched(Windows::ApplicationModel::Activation::LaunchActivatedEventArgs ^ e) override;

private:
    OpenGLESPage ^ mPage;
    OpenGLES mOpenGLES;
};
}
