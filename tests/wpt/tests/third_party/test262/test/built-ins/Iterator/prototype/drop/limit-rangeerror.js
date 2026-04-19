// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Throws a RangeError exception when limit argument is NaN or less than 0.
info: |
  %Iterator.prototype%.drop ( limit )

  3. If numLimit is NaN, throw a RangeError exception.
  4. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
  5. If integerLimit < 0, throw a RangeError exception.

features: [iterator-helpers]
---*/
let iterator = (function* () {})();

iterator.drop(0);
iterator.drop(-0.5);
iterator.drop(null);

assert.throws(RangeError, () => {
  iterator.drop(-1);
});

assert.throws(RangeError, () => {
  iterator.drop();
});

assert.throws(RangeError, () => {
  iterator.drop(undefined);
});

assert.throws(RangeError, () => {
  iterator.drop(NaN);
});
