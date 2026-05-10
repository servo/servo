// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tojson
description: Basic behavior for toJSON
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainYearMonth(1972, 1), "1972-01"],
  [new Temporal.PlainYearMonth(1972, 12), "1972-12"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});

for (const [yearMonth, expected] of tests) {
  assert.sameValue(yearMonth.toJSON(), expected, "toJSON without argument");
  assert.sameValue(yearMonth.toJSON(options), expected, "toJSON with argument");
}
