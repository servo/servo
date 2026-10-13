// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  skippedElements values that are not Numbers throw TypeError and close the iterator
features: [iterator-includes]
---*/

function assertTypeErrorAndClosed(skippedElements, label) {
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

  assert.throws(TypeError, function() {
    iterator.includes(0, skippedElements);
  }, label + ': throws TypeError');

  assert.sameValue(closed, true, label + ': iterator closed');
}

assertTypeErrorAndClosed(true, 'boolean');
assertTypeErrorAndClosed(null, 'null');
assertTypeErrorAndClosed('1', 'string');
assertTypeErrorAndClosed({}, 'object');
assertTypeErrorAndClosed([], 'array');
assertTypeErrorAndClosed(Symbol(), 'Symbol');
assertTypeErrorAndClosed(1n, 'BigInt');
