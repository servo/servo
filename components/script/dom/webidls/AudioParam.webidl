/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#dom-audioparam
 */

enum AutomationRate {
  "a-rate",
  "k-rate"
};

[Exposed=Window]
interface AudioParam {
             attribute float value;
             [SetterThrows] attribute AutomationRate automationRate;
    readonly attribute float defaultValue;
    readonly attribute float minValue;
    readonly attribute float maxValue;
    [Throws] AudioParam setValueAtTime(float value, double startTime);
    [Throws] AudioParam linearRampToValueAtTime(float value, double endTime);
    [Throws] AudioParam exponentialRampToValueAtTime(float value, double endTime);
    [Throws] AudioParam setTargetAtTime(float target,
                               double startTime,
                               float timeConstant);
    [Throws] AudioParam setValueCurveAtTime(sequence<float> values,
                                   double startTime,
                                   double duration);
    [Throws] AudioParam cancelScheduledValues(double cancelTime);
    [Throws] AudioParam cancelAndHoldAtTime(double cancelTime);
};
