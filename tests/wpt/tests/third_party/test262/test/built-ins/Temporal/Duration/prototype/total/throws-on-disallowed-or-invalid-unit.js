// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: total() throws on disallowed or invalid unit
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);

// Object param
[
  "era",
  "nonsense"
].forEach(unit => {
  assert.throws(RangeError, () => d.total({ unit }));
});

// String param
[
  "era",
  "nonsense"
].forEach(unit => {
  assert.throws(RangeError, () => d.total(unit));
});
