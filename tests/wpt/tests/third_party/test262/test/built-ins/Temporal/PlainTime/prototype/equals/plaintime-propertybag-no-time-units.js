// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: Missing time units in property bag default to 0
features: [Temporal]
---*/

const instance = new Temporal.PlainTime(0, 30, 0, 0, 0, 0);

const props = {};
assert.throws(TypeError, () => instance.equals(props), "TypeError if no properties are present");

props.minute = 30;
const result = instance.equals(props);
assert.sameValue(result, true, "missing time units default to 0");
