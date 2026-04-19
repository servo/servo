// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.plaintimeiso
description: PlainDateTime.toPlainTime is not observably called
includes: [compareArray.js]
features: [Temporal]
---*/

const actual = [];
const expected = [];

Object.defineProperty(Temporal.PlainDateTime.prototype, "toPlainTime", {
  get() {
    actual.push("get Temporal.PlainDateTime.prototype.toPlainTime");
    return function() {
      actual.push("call Temporal.PlainDateTime.prototype.toPlainTime");
    };
  },
});

const result = Temporal.Now.plainTimeISO("UTC");
assert.sameValue(result instanceof Temporal.PlainTime, true);

assert.compareArray(actual, expected);
