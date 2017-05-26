/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvr/#interface-navigator
[NoInterfaceObject]
interface VR {
  Promise<sequence<VRDisplay>> getDisplays();
  //readonly attribute FrozenArray<VRDisplay> activeVRDisplays;
};
