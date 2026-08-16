// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.drop
description: >
  Throws a RangeError exception when limit argument is NaN, less than 0, or
  finite and greater than Number.MAX_SAFE_INTEGER.
info: |
  Iterator.prototype.drop ( limit )

  6. If numLimit is NaN, then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).
  7. If numLimit is finite and numLimit > 𝔽(2**53 - 1), then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).
  8. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
  9. If integerLimit < 0, then
     a. Let error be ThrowCompletion(a newly created RangeError object).
     b. Return ? IteratorClose(iterated, error).

features: [iterator-helpers]
---*/
let iterator = (function* () {})();

iterator.drop(0);
iterator.drop(-0.5);
iterator.drop(null);
iterator.drop(Number.MAX_SAFE_INTEGER);
iterator.drop(Infinity);

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

assert.throws(RangeError, () => {
  iterator.drop(Number.MAX_SAFE_INTEGER + 1);
});
