// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: Missing time units in property bag default to 0
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 1, 1);

const props = {};
assert.throws(TypeError, () => instance.toPlainDateTime(props), "TypeError if no properties are present");

props.minute = 30;
const result = instance.toPlainDateTime(props);
TemporalHelpers.assertPlainDateTime(result, 2000, 1, "M01", 1, 0, 30, 0, 0, 0, 0, "missing time units default to 0");
