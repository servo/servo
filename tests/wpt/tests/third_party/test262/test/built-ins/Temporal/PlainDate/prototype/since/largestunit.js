// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Specify behavior of PlainDate.since when largest specified unit is years or months.
includes: [temporalHelpers.js]
features: [Temporal]
---*/
const pd1 = new Temporal.PlainDate(2020, 11, 1);
const pd2 = new Temporal.PlainDate(2021, 11, 30);
TemporalHelpers.assertDuration(pd2.since(pd1), 0, 0, 0, 394, 0, 0, 0, 0, 0, 0, 'does not include higher units than necessary (largest unit unspecified)');
TemporalHelpers.assertDuration(pd2.since(pd1, { largestUnit: 'months' }), 0, 12, 0, 29, 0, 0, 0, 0, 0, 0,  'does not include higher units than necessary (largest unit is months)');
TemporalHelpers.assertDuration(pd2.since(pd1, { largestUnit: 'years' }), 1, 0, 0, 29, 0, 0, 0, 0, 0, 0, 'does not include higher units than necessary (largest unit is years)');
