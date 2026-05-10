// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tojson
description: Basic behavior for toJSON
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainDate(1976, 2, 4), "1976-02-04"],
  [new Temporal.PlainDate(1976, 11, 18), "1976-11-18"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});
for (const [datetime, expected] of tests) {
  assert.sameValue(datetime.toJSON(), expected, "toJSON without argument");
  assert.sameValue(datetime.toJSON(options), expected, "toJSON with argument");
}
