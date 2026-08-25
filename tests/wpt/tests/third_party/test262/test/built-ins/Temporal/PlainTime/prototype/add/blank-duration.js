// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const t = new Temporal.PlainTime(14, 1);
const blank = new Temporal.Duration();
const result = t.add(blank);
TemporalHelpers.assertPlainTime(result, 14, 1, 0, 0, 0, 0, "result is unchanged");
