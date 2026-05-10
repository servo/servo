// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Does not throw a RangeError if only one of era/eraYear fields is present
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const base = { year: 2000, month: 5, day: 2, era: 'ce' };
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from(base), 2000, 5, 'M05', 2);

const base2 = { year: 2000, month: 5, day: 2, eraYear: 1 };
TemporalHelpers.assertPlainDate(Temporal.PlainDate.from(base2), 2000, 5, 'M05', 2);
