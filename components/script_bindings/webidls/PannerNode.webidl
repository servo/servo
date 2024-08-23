/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#pannernode
 */

dictionary PannerOptions : AudioNodeOptions {
  PanningModelType panningModel = "equalpower";
  DistanceModelType distanceModel = "inverse";
  float positionX = 0;
  float positionY = 0;
  float positionZ = 0;
  float orientationX = 1;
  float orientationY = 0;
  float orientationZ = 0;
  double refDistance = 1;
  double maxDistance = 10000;
  double rolloffFactor = 1;
  double coneInnerAngle = 360;
  double coneOuterAngle = 360;
  double coneOuterGain = 0;
};

enum DistanceModelType {
  "linear",
  "inverse",
  "exponential"
};

enum PanningModelType {
    "equalpower",
    "HRTF"
};

[Exposed=Window]
interface PannerNode : AudioNode {
  [Throws] constructor(BaseAudioContext context, optional PannerOptions options = {});
  attribute PanningModelType panningModel;
  readonly attribute AudioParam positionX;
  readonly attribute AudioParam positionY;
  readonly attribute AudioParam positionZ;
  readonly attribute AudioParam orientationX;
  readonly attribute AudioParam orientationY;
  readonly attribute AudioParam orientationZ;
  attribute DistanceModelType distanceModel;
  [SetterThrows] attribute double refDistance;
  [SetterThrows] attribute double maxDistance;
  [SetterThrows] attribute double rolloffFactor;
  attribute double coneInnerAngle;
  attribute double coneOuterAngle;
  [SetterThrows] attribute double coneOuterGain;
  undefined setPosition (float x, float y, float z);
  undefined setOrientation (float x, float y, float z);
};
