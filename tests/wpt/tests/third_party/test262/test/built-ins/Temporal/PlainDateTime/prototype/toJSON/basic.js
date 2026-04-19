// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tojson
description: Basic behavior for toJSON
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainDateTime(1976, 2, 4, 5, 3, 1), "1976-02-04T05:03:01"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23), "1976-11-18T15:23:00"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30), "1976-11-18T15:23:30"],
  [new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 400), "1976-11-18T15:23:30.1234"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});
for (const [datetime, expected] of tests) {
  assert.sameValue(datetime.toJSON(), expected, "toJSON without argument");
  assert.sameValue(datetime.toJSON(options), expected, "toJSON with argument");
}
