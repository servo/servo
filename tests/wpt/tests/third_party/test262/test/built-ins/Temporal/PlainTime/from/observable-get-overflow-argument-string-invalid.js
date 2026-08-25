// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: overflow property is not extracted with ISO-invalid string argument.
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

let actual = [];
const object = {
  get overflow() {
    actual.push("get overflow");
    return TemporalHelpers.toPrimitiveObserver(actual, "constrain", "overflow");
  }
};

assert.throws(RangeError, () => Temporal.PlainTime.from("24:60", object));
assert.compareArray(actual, [], "options read after ISO string parsing");
