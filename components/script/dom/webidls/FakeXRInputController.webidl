/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr-test-api/#fakexrinputcontroller

[Exposed=Window, Pref="dom.webxr.test"]
interface FakeXRInputController {
  undefined setHandedness(XRHandedness handedness);
  undefined setTargetRayMode(XRTargetRayMode targetRayMode);
  undefined setProfiles(sequence<DOMString> profiles);
  [Throws] undefined setGripOrigin(FakeXRRigidTransformInit gripOrigin, optional boolean emulatedPosition = false);
  undefined clearGripOrigin();
  [Throws] undefined setPointerOrigin(
    FakeXRRigidTransformInit pointerOrigin,
    optional boolean emulatedPosition = false
  );

  undefined disconnect();
  undefined reconnect();

  undefined startSelection();
  undefined endSelection();
  undefined simulateSelect();

  // void setSupportedButtons(sequence<FakeXRButtonStateInit> supportedButtons);
  // void updateButtonState(FakeXRButtonStateInit buttonState);
};

dictionary FakeXRInputSourceInit {
  required XRHandedness handedness;
  required XRTargetRayMode targetRayMode;
  required FakeXRRigidTransformInit pointerOrigin;
  required sequence<DOMString> profiles;
  boolean selectionStarted = false;
  boolean selectionClicked = false;
  sequence<FakeXRButtonStateInit> supportedButtons;
  FakeXRRigidTransformInit gripOrigin;
};

enum FakeXRButtonType {
  "grip",
  "touchpad",
  "thumbstick",
  "optional-button",
  "optional-thumbstick"
};

dictionary FakeXRButtonStateInit {
  required FakeXRButtonType buttonType;
  required boolean pressed;
  required boolean touched;
  required float pressedValue;
  float xValue = 0.0;
  float yValue = 0.0;
};
