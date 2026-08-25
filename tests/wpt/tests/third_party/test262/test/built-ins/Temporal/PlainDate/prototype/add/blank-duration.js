// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Behaviour with blank duration
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const d = new Temporal.PlainDate(2025, 8, 22);
const blank = new Temporal.Duration();
const result = d.add(blank);
TemporalHelpers.assertPlainDate(result, 2025, 8, "M08", 22, "result is unchanged");
