// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tojson
description: Basic behavior for toJSON
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainMonthDay(1, 1), "01-01"],
  [new Temporal.PlainMonthDay(12, 31), "12-31"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});

for (const [monthDay, expected] of tests) {
  assert.sameValue(monthDay.toJSON(), expected, "toJSON without argument");
  assert.sameValue(monthDay.toJSON(options), expected, "toJSON with argument");
}
