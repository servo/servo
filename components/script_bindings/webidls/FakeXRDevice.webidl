/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://github.com/immersive-web/webxr-test-api/

[Exposed=Window, Pref="dom_webxr_test"]
interface FakeXRDevice {
  // Sets the values to be used for subsequent requestAnimationFrame() callbacks.
  [Throws] undefined setViews(sequence<FakeXRViewInit> views, optional sequence<FakeXRViewInit> secondaryViews);

  // behaves as if device was disconnected
  Promise<undefined> disconnect();

  [Throws] undefined setViewerOrigin(FakeXRRigidTransformInit origin, optional boolean emulatedPosition = false);
  undefined clearViewerOrigin();
  [Throws] undefined setFloorOrigin(FakeXRRigidTransformInit origin);
  undefined clearFloorOrigin();
  [Throws] undefined setBoundsGeometry(sequence<FakeXRBoundsPoint> boundsCoodinates);
  undefined simulateResetPose();

  // Simulates devices focusing and blurring sessions.
  undefined simulateVisibilityChange(XRVisibilityState state);

  [Throws] FakeXRInputController simulateInputSourceConnection(FakeXRInputSourceInit init);

  // Hit test extensions:
  [Throws] undefined setWorld(FakeXRWorldInit world);
  undefined clearWorld();

  // Depth sensing extensions:
  // undefined setDepthSensingData(FakeXRDepthSensingDataInit depthSensingData);
  // undefined clearDepthSensingData();
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

  FakeXRFieldOfViewInit fieldOfView;
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

dictionary FakeXRFieldOfViewInit {
  required float upDegrees;
  required float downDegrees;
  required float leftDegrees;
  required float rightDegrees;
};

// hit testing
dictionary FakeXRWorldInit {
  required sequence<FakeXRRegionInit> hitTestRegions;
};


dictionary FakeXRRegionInit {
  required sequence<FakeXRTriangleInit> faces;
  required FakeXRRegionType type;
};


dictionary FakeXRTriangleInit {
  required sequence<DOMPointInit> vertices;  // size = 3
};


enum FakeXRRegionType {
  "point",
  "plane",
  "mesh"
};
