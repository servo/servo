// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tojson
description: Basic behavior for toJSON
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainTime(5, 3, 1), "05:03:01"],
  [new Temporal.PlainTime(15, 23), "15:23:00"],
  [new Temporal.PlainTime(15, 23, 30), "15:23:30"],
  [new Temporal.PlainTime(15, 23, 30, 123, 400), "15:23:30.1234"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});
for (const [time, expected] of tests) {
  assert.sameValue(time.toJSON(), expected, "toJSON without argument");
  assert.sameValue(time.toJSON(options), expected, "toJSON with argument");
}
