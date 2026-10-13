// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Finite skippedElements values greater than Number.MAX_SAFE_INTEGER throw RangeError and close the iterator
features: [iterator-includes]
---*/

function assertRangeErrorAndClosed(skippedElements, label) {
  let closed = false;
  let iterator = {
    __proto__: Iterator.prototype,
    get next() {
      throw new Test262Error('next should not be read');
    },
    return() {
      closed = true;
      return {};
    },
  };

  assert.throws(RangeError, function() {
    iterator.includes(0, skippedElements);
  }, label + ': throws RangeError');

  assert.sameValue(closed, true, label + ': iterator closed');
}

assertRangeErrorAndClosed(Number.MAX_SAFE_INTEGER + 1, 'Number.MAX_SAFE_INTEGER + 1');
assertRangeErrorAndClosed(Number.MAX_SAFE_INTEGER + 3, 'Number.MAX_SAFE_INTEGER + 3');
