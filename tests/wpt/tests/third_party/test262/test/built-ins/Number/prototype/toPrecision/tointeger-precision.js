// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  ToInteger(precision) operations
info: |
  Number.prototype.toPrecision ( precision )

  [...]
  3. Let p be ? ToInteger(precision).
  [...]
---*/

assert.sameValue((123.456).toPrecision(1.1), "1e+2", "1.1");
assert.sameValue((123.456).toPrecision(1.9), "1e+2", "1.9");

assert.sameValue((123.456).toPrecision(true), "1e+2", "true");

assert.sameValue((123.456).toPrecision("2"), "1.2e+2", "string");

assert.sameValue((123.456).toPrecision([2]), "1.2e+2", "[2]");
