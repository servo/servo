/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://github.com/immersive-web/webxr-test-api/

[Exposed=Window, Pref="dom.webxr.test"]
interface FakeXRDevice {
  // Sets the values to be used for subsequent
  // requestAnimationFrame() callbacks.
  [Throws] void setViews(sequence<FakeXRViewInit> views);

  // // behaves as if device was disconnected
  // Promise<void> disconnect();

  // Sets the origin of the viewer
  [Throws] void setViewerOrigin(FakeXRRigidTransformInit origin, optional boolean emulatedPosition = false);

  // // Simulates devices focusing and blurring sessions.
  // void simulateVisibilityChange(XRVisibilityState);

  // void setBoundsGeometry(sequence<FakeXRBoundsPoint> boundsCoodinates);
  // // Sets eye level used for calculating floor-level spaces
  // void setEyeLevel(float eyeLevel);


  // Promise<FakeXRInputController>
  //     simulateInputSourceConnection(FakeXRInputSourceInit);
};

// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport
dictionary FakeXRViewInit {
  required XREye eye;
  // https://immersive-web.github.io/webxr/#view-projection-matrix
  required sequence<float> projectionMatrix;
  // https://immersive-web.github.io/webxr/#view-offset
  required FakeXRRigidTransformInit viewOffset;
  // https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport
  required FakeXRDeviceResolution resolution;
};

// https://immersive-web.github.io/webxr/#xrviewport
dictionary FakeXRDeviceResolution {
    required long width;
    required long height;
};

dictionary FakeXRBoundsPoint {
  double x; double z;
};

dictionary FakeXRRigidTransformInit {
    required sequence<float> position;
    required sequence<float> orientation;
};
