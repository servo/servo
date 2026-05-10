// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: Number.prototype.toPrecision permits fractionDigits from 1 to 100
info: |
  Number.prototype.toPrecision ( fractionDigits )

  ...
  8. If _p_ &lt; 1 or _p_ &gt; 100, throw a *RangeError* exception.
  ...
---*/

assert.sameValue((3).toPrecision(1), "3");
assert.throws(RangeError, () => (3).toPrecision(0));
assert.throws(RangeError, () => (3).toPrecision(-10));

assert.sameValue((3).toPrecision(100), "3.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
assert.throws(RangeError, () => (3).toPrecision(101));
