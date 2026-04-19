// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Non-iterable input does not use Array.prototype
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
const arrayIterator = [].values();
const IntrinsicArrayIteratorPrototype =
Object.getPrototypeOf(arrayIterator);
const intrinsicArrayIteratorPrototypeNext =
IntrinsicArrayIteratorPrototype.next;

try {
// Temporarily mutate the array iterator prototype to have an invalid
// “next” method. Just like Array.from, the fromAsync function should
// still work on non-iterable arraylike arguments.
IntrinsicArrayIteratorPrototype.next = function fakeNext () {
  throw new Test262Error(
    'This fake next function should not be called; ' +
    'instead, each element should have been directly accessed.',
  );
};

const expected = [ 0, 1, 2 ];
const input = {
  length: 3,
  0: 0,
  1: 1,
  2: 2,
};
const output = await Array.fromAsync(input);
assert.compareArray(output, expected);
}

finally {
// Reset the intrinsic array iterator
IntrinsicArrayIteratorPrototype.next =
  intrinsicArrayIteratorPrototypeNext;
}
});
