/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://github.com/immersive-web/webxr-test-api/

[Exposed=Window, Pref="dom.webxr.test"]
interface XRTest {
  // Simulates connecting a device to the system.
  // Used to instantiate a fake device for use in tests.
  Promise<FakeXRDeviceController> simulateDeviceConnection(FakeXRDeviceInit init);

  // // Simulates a device being disconnected from the system.
  // Promise<void> simulateDeviceDisconnection(XRDevice);

  // // Simulates a user activation (aka user gesture) for the current scope.
  // // The activation is only guaranteed to be valid in the provided function and only applies to WebXR
  // // Device API methods.
  // void simulateUserActivation(Function);
};

dictionary FakeXRDeviceInit {
    // TODO: Subject to change to match spec changes.
    required boolean supportsImmersive;
};
