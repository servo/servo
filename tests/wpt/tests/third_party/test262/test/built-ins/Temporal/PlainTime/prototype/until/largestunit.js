// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: PlainTime.until with various largestUnit values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/
const fourFortyEight = new Temporal.PlainTime(4, 48, 55);
const elevenFiftyNine = new Temporal.PlainTime(11, 59, 58);
TemporalHelpers.assertDuration(fourFortyEight.until(elevenFiftyNine), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0, "does not include higher units than necessary (largest unit unspecified)");
TemporalHelpers.assertDuration(fourFortyEight.until(elevenFiftyNine, { largestUnit: "auto" }), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0,  "does not include higher units than necessary (largest unit is auto)");
TemporalHelpers.assertDuration(fourFortyEight.until(elevenFiftyNine, { largestUnit: "hours" }), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0,  "does not include higher units than necessary (largest unit is hours)");
TemporalHelpers.assertDuration(fourFortyEight.until(elevenFiftyNine, { largestUnit: "minutes" }), 0, 0, 0, 0, 0, 431, 3, 0, 0, 0, "does not include higher units than necessary (largest unit is minutes)");
TemporalHelpers.assertDuration(fourFortyEight.until(elevenFiftyNine, { largestUnit: "seconds" }), 0, 0, 0, 0, 0, 0, 25863, 0, 0, 0, "does not include higher units than necessary (largest unit is seconds)");
