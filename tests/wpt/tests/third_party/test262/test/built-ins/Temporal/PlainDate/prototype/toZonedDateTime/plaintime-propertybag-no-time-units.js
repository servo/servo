// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Missing time units in property bag default to 0
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 1, 1);

const props = {};
assert.throws(TypeError, () => instance.toZonedDateTime({ plainTime: props, timeZone: "UTC" }), "TypeError if no properties are present");

props.minute = 30;
const result = instance.toZonedDateTime({ plainTime: props, timeZone: "UTC" });
assert.sameValue(result.epochNanoseconds, 946686600_000_000_000n, "missing time units default to 0");
