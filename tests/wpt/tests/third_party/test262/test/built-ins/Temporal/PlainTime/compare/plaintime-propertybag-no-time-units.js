// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: Missing time units in property bag default to 0
features: [Temporal]
---*/

const props = {};
assert.throws(TypeError, () => Temporal.PlainTime.compare(props, new Temporal.PlainTime(0, 30)), "TypeError if no properties are present");

props.minute = 30;
const result = Temporal.PlainTime.compare(props, new Temporal.PlainTime(0, 30));
assert.sameValue(result, 0, "missing time units default to 0");
