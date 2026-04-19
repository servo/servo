// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: Missing time units in property bag default to 0
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 1, 1, 12, 30, 45, 123, 456, 789);

const props = {};
assert.throws(TypeError, () => instance.withPlainTime(props), "TypeError if no properties are present");

props.minute = 30;
const result = instance.withPlainTime(props);
TemporalHelpers.assertPlainDateTime(result, 2000, 1, "M01", 1, 0, 30, 0, 0, 0, 0, "missing time units default to 0");
