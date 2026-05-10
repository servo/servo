// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: Missing time units in property bag default to 0
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

const props = {};
assert.throws(TypeError, () => instance.withPlainTime(props), "TypeError if no properties are present");

props.minute = 30;
const result = instance.withPlainTime(props);
assert.sameValue(result.epochNanoseconds, 999995400_000_000_000n, "missing time units default to 0");
