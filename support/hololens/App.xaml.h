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
