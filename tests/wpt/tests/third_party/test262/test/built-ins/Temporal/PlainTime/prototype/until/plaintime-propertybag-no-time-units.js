// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: Missing time units in property bag default to 0
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainTime(1, 0, 0, 0, 0, 1);

const props = {};
assert.throws(TypeError, () => instance.until(props), "TypeError if no properties are present");

props.minute = 30;
const result = instance.until(props);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, -30, 0, 0, 0, -1, "missing time units default to 0");
