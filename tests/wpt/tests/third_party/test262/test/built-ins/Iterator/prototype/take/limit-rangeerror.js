// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Throws a RangeError exception when limit argument is NaN or less than 0.
info: |
  %Iterator.prototype%.take ( limit )

  4. If numLimit is NaN, throw a RangeError exception.
  5. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
  6. If integerLimit < 0, throw a RangeError exception.

features: [iterator-helpers]
---*/
let iterator = (function* () {})();

iterator.take(0);
iterator.take(-0.5);
iterator.take(null);

assert.throws(RangeError, () => {
  iterator.take(-1);
});

assert.throws(RangeError, () => {
  iterator.take();
});

assert.throws(RangeError, () => {
  iterator.take(undefined);
});

assert.throws(RangeError, () => {
  iterator.take(NaN);
});
