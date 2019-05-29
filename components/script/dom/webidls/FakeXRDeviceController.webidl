/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://github.com/immersive-web/webxr-test-api/

[Exposed=Window, Pref="dom.webxr.test"]
interface FakeXRDeviceController {
  // Creates and attaches a XRFrameOfReference of the type specified to the device.
  // void setFrameOfReference(XRFrameOfReferenceType,  FakeXRFrameOfReferenceInit);

  // // Sets the values to be used for subsequent
  // // requestAnimationFrame() callbacks.
  // void setViews(Array<FakeXRViewInit> views);

  // void setViewerOrigin(FakeXRRigidTransform origin);

  // Simulates the user activating the reset pose on a device.
  // void simulateResetPose();

  // Simulates the platform ending the sessions.
  // void simulateForcedEndSessions();

  // Simulates devices focusing and blurring sessions.
  // void simulateBlurSession(XRSession);
  // void simulateFocusSession(XRSession);

  // void setBoundsGeometry(Array<FakeXRBoundsPoint> boundsCoodinates)l

  // Promise<FakeXRInputSourceController>
  //     simulateInputSourceConnection(FakeXRInputSourceInit);
};

dictionary FakeXRViewInit {
  required XREye eye;
  // https://immersive-web.github.io/webxr/#view-projection-matrix
  required sequence<float> projectionMatrix;
  // https://immersive-web.github.io/webxr/#view-offset
  required FakeXRRigidTransform viewOffset;
};

dictionary FakeXRBoundsPoint {
  double x; double z;
};

dictionary FakeXRRigidTransform {
    required sequence<float> position;
    required sequence<float> orientation;
};
