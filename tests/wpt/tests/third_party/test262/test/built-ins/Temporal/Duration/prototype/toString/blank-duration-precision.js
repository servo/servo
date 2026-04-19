// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: >
    Precision is handled correctly for blank durations, whether specified by
    fractionalSecondDigits or smallestUnit
features: [Temporal]
---*/

const blank = new Temporal.Duration();

assert.sameValue(blank.toString({ fractionalSecondDigits: "auto" }), "PT0S");
assert.sameValue(blank.toString({ fractionalSecondDigits: 0 }), "PT0S");
assert.sameValue(blank.toString({ fractionalSecondDigits: 2 }), "PT0.00S");
assert.sameValue(blank.toString({ fractionalSecondDigits: 9 }), "PT0.000000000S");

assert.sameValue(blank.toString({ smallestUnit: "seconds" }), "PT0S");
assert.sameValue(blank.toString({ smallestUnit: "milliseconds" }), "PT0.000S");
assert.sameValue(blank.toString({ smallestUnit: "microseconds" }), "PT0.000000S");
assert.sameValue(blank.toString({ smallestUnit: "nanoseconds" }), "PT0.000000000S");
