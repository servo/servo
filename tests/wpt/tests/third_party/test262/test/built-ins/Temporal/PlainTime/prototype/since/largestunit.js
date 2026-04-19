// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: PlainTime.since with various largestUnit values.
includes: [temporalHelpers.js]
features: [Temporal]
---*/
const fourFortyEight = new Temporal.PlainTime(4, 48, 55);
const elevenFiftyNine = new Temporal.PlainTime(11, 59, 58);
TemporalHelpers.assertDuration(elevenFiftyNine.since(fourFortyEight), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0, 'does not include higher units than necessary (largest unit unspecified)');
TemporalHelpers.assertDuration(elevenFiftyNine.since(fourFortyEight, { largestUnit: 'auto' }), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0,  'does not include higher units than necessary (largest unit is auto)');
TemporalHelpers.assertDuration(elevenFiftyNine.since(fourFortyEight, { largestUnit: 'hours' }), 0, 0, 0, 0, 7, 11, 3, 0, 0, 0,  'does not include higher units than necessary (largest unit is hours)');
TemporalHelpers.assertDuration(elevenFiftyNine.since(fourFortyEight, { largestUnit: 'minutes' }), 0, 0, 0, 0, 0, 431, 3, 0, 0, 0, 'does not include higher units than necessary (largest unit is minutes)');
TemporalHelpers.assertDuration(elevenFiftyNine.since(fourFortyEight, { largestUnit: 'seconds' }), 0, 0, 0, 0, 0, 0, 25863, 0, 0, 0, 'does not include higher units than necessary (largest unit is seconds)');
