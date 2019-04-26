/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#audiolistener
 */

[Exposed=Window]
interface AudioListener {
  readonly attribute AudioParam positionX;
  readonly attribute AudioParam positionY;
  readonly attribute AudioParam positionZ;
  readonly attribute AudioParam forwardX;
  readonly attribute AudioParam forwardY;
  readonly attribute AudioParam forwardZ;
  readonly attribute AudioParam upX;
  readonly attribute AudioParam upY;
  readonly attribute AudioParam upZ;
  [Throws] AudioListener setPosition (float x, float y, float z);
  [Throws] AudioListener setOrientation (float x, float y, float z, float xUp, float yUp, float zUp);
};
