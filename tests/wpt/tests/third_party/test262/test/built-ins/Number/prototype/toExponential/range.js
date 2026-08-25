// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: Number.prototype.toExponential permits fractionDigits from 0 to 100
info: |
  Number.prototype.toExponential ( fractionDigits )

  ...
  8. If _p_ &lt; 0 or _p_ &gt; 100, throw a *RangeError* exception.
  ...
---*/

assert.sameValue((3).toExponential(0), "3e+0");
assert.throws(RangeError, () => (3).toExponential(-1));

assert.sameValue((3).toExponential(100), "3.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e+0");
assert.throws(RangeError, () => (3).toExponential(101));
