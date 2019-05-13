/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/mediacapture-main/#dom-mediadevices

[Exposed=Window,
SecureContext, Pref="dom.mediadevices.enabled"]
interface MediaDevices : EventTarget {
    //                attribute EventHandler ondevicechange;
    // Promise<sequence<MediaDeviceInfo>> enumerateDevices();
};

partial interface Navigator {
    // [SameObject, SecureContext]
    [Pref="dom.mediadevices.enabled"] readonly        attribute MediaDevices mediaDevices;
};

partial interface MediaDevices {
    // MediaTrackSupportedConstraints getSupportedConstraints();
    Promise<MediaStream> getUserMedia(optional MediaStreamConstraints constraints);
};


dictionary MediaStreamConstraints {
        (boolean or MediaTrackConstraints) video = false;
        (boolean or MediaTrackConstraints) audio = false;
};

dictionary DoubleRange {
             double max;
             double min;
};

dictionary ConstrainDoubleRange : DoubleRange {
             double exact;
             double ideal;
};

dictionary ULongRange {
             [Clamp] unsigned long max;
             [Clamp] unsigned long min;
};

dictionary ConstrainULongRange : ULongRange {
             [Clamp] unsigned long exact;
             [Clamp] unsigned long ideal;
};

// dictionary ConstrainBooleanParameters {
//              boolean exact;
//              boolean ideal;
// };

// dictionary ConstrainDOMStringParameters {
//              (DOMString or sequence<DOMString>) exact;
//              (DOMString or sequence<DOMString>) ideal;
// };

dictionary MediaTrackConstraints : MediaTrackConstraintSet {
             sequence<MediaTrackConstraintSet> advanced;
};

typedef ([Clamp] unsigned long or ConstrainULongRange) ConstrainULong;
typedef (double or ConstrainDoubleRange) ConstrainDouble;
// typedef (boolean or ConstrainBooleanParameters) ConstrainBoolean;
// typedef (DOMString or sequence<DOMString> or ConstrainDOMStringParameters) ConstrainDOMString;

dictionary MediaTrackConstraintSet {
             ConstrainULong width;
             ConstrainULong height;
             ConstrainDouble aspectRatio;
             ConstrainDouble frameRate;
             // ConstrainDOMString facingMode;
             // ConstrainDOMString resizeMode;
             // ConstrainDouble volume;
             ConstrainULong sampleRate;
             // ConstrainULong sampleSize;
             // ConstrainBoolean echoCancellation;
             // ConstrainBoolean autoGainControl;
             // ConstrainBoolean noiseSuppression;
             // ConstrainDouble latency;
             // ConstrainULong channelCount;
             // ConstrainDOMString deviceId;
             // ConstrainDOMString groupId;
};
