// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Missing time units in property bag default to 0
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const props = {};
assert.throws(TypeError, () => Temporal.PlainTime.from(props), "TypeError if at least one property is not present");

props.minute = 30;
const result = Temporal.PlainTime.from(props);
TemporalHelpers.assertPlainTime(result, 0, 30, 0, 0, 0, 0, "missing time units default to 0");
