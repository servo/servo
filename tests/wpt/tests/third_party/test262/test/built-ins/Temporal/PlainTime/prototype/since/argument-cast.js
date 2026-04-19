// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Casts the argument
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
TemporalHelpers.assertDuration(plainTime.since("16:34"),
  0, 0, 0, 0, /* hours = */ -1, /* minutes = */ -10, /* seconds = */ -29, -876, -543, -211, "string");
TemporalHelpers.assertDuration(plainTime.since({ hour: 16, minute: 34 }),
  0, 0, 0, 0, /* hours = */ -1, /* minutes = */ -10, /* seconds = */ -29, -876, -543, -211, "object");

assert.throws(TypeError, () => plainTime.since({}), "empty");
assert.throws(TypeError, () => plainTime.since({ minutes: 30 }), "only plural 'minutes'");
