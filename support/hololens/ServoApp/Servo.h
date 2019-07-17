/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"

extern "C" {
#include <simpleservo.h>
}

class Servo {
public:
  Servo(GLsizei width, GLsizei height);
  ~Servo();
  void PerformUpdates();
  void Click(float, float);
  void SetSize(GLsizei width, GLsizei height);
  void SetBatchMode(bool);
  void GoBack();
  void GoForward();

  // Static lambas called by Servo callbacks.

  // Will be called from any thead
  static std::function<void()> sWakeUp;
  // Will be called from GL thread
  static std::function<void()> sFlush;
  static std::function<void()> sMakeCurrent;
  static std::function<void(std::wstring const &)> sOnAlert;
  static std::function<void(std::wstring const &)> sOnTitleChanged;
  static std::function<void(std::wstring const &)> sOnURLChanged;
  static bool sAnimating;

private:
  GLsizei mWindowWidth;
  GLsizei mWindowHeight;
  bool mAnimating;
};
