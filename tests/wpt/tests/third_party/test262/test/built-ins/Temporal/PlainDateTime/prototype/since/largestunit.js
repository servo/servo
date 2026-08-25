// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Specify behavior of PlainDateTime.since when largest specified unit is years or months.
includes: [temporalHelpers.js]
features: [Temporal]
---*/
const lastFeb20 = new Temporal.PlainDateTime(2020, 2, 20, 5, 45, 20);
const lastFeb21 = new Temporal.PlainDateTime(2021, 2, 21, 17, 18, 57);
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20), 0, 0, 0, 367, 11, 33, 37, 0, 0, 0, 'does not include higher units than necessary (largest unit unspecified)');
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20, { largestUnit: 'months' }), 0, 12, 0, 1, 11, 33, 37, 0, 0, 0,  'does not include higher units than necessary (largest unit is months)');
TemporalHelpers.assertDuration(lastFeb21.since(lastFeb20, { largestUnit: 'years' }), 1, 0, 0, 1, 11, 33, 37, 0, 0, 0, 'does not include higher units than necessary (largest unit is years)');
