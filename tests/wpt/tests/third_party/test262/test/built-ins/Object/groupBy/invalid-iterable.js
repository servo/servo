// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy with a nullish Symbol.iterator throws
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  4. Let iteratorRecord be ? GetIterator(items).

  ...
features: [array-grouping]
---*/

const throws = function () {
  throw new Test262Error('callback function should not be called')
};

function makeIterable(obj, iteratorFn) {
  obj[Symbol.iterator] = iteratorFn;
  return obj;
}

assert.throws(TypeError, function () {
  Object.groupBy(makeIterable({}, undefined), throws);
}, 'undefined Symbol.iterator');

assert.throws(TypeError, function () {
  Object.groupBy(makeIterable({}, null), throws);
}, 'null Symbol.iterator');
