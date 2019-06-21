#pragma once

#include "pch.h"

namespace hlservo {
class Servo {
public:
    Servo(GLsizei width, GLsizei height);
    ~Servo();
    void PerformUpdates();
    void SetSize(GLsizei width, GLsizei height);

    // Static lambas called by Servo callbacks.

    // Will be called from any thead
    static std::function<void()> sWakeUp;
    // Will be called from GL thread
    static std::function<void()> sFlush;
    static std::function<void()> sMakeCurrent;
    static bool sAnimating;

private:
    GLsizei mWindowWidth;
    GLsizei mWindowHeight;
    bool mAnimating;
};
}
