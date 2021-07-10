/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md

[SecureContext, Exposed=Window, Pref="dom.webxr.hands.enabled"]
interface XRHand {
    readonly attribute long length;
    getter XRJointSpace(unsigned long index);

    const unsigned long WRIST = 0;
    const unsigned long THUMB_METACARPAL = 1;
    const unsigned long THUMB_PHALANX_PROXIMAL = 2;
    const unsigned long THUMB_PHALANX_DISTAL = 3;
    const unsigned long THUMB_PHALANX_TIP = 4;

    const unsigned long INDEX_METACARPAL = 5;
    const unsigned long INDEX_PHALANX_PROXIMAL = 6;
    const unsigned long INDEX_PHALANX_INTERMEDIATE = 7;
    const unsigned long INDEX_PHALANX_DISTAL = 8;
    const unsigned long INDEX_PHALANX_TIP = 9;

    const unsigned long MIDDLE_METACARPAL = 10;
    const unsigned long MIDDLE_PHALANX_PROXIMAL = 11;
    const unsigned long MIDDLE_PHALANX_INTERMEDIATE = 12;
    const unsigned long MIDDLE_PHALANX_DISTAL = 13;
    const unsigned long MIDDLE_PHALANX_TIP = 14;

    const unsigned long RING_METACARPAL = 15;
    const unsigned long RING_PHALANX_PROXIMAL = 16;
    const unsigned long RING_PHALANX_INTERMEDIATE = 17;
    const unsigned long RING_PHALANX_DISTAL = 18;
    const unsigned long RING_PHALANX_TIP = 19;

    const unsigned long LITTLE_METACARPAL = 20;
    const unsigned long LITTLE_PHALANX_PROXIMAL = 21;
    const unsigned long LITTLE_PHALANX_INTERMEDIATE = 22;
    const unsigned long LITTLE_PHALANX_DISTAL = 23;
    const unsigned long LITTLE_PHALANX_TIP = 24;
};
