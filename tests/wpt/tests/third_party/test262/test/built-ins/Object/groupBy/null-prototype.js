// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy returns a null prototype object
info: |
  Object.groupBy ( items, callbackfn )

  ...

  2. Let obj be OrdinaryObjectCreate(null).
  ...
  4. Return obj.

  ...
features: [array-grouping]
---*/

const array = [1, 2, 3];

const obj = Object.groupBy(array, function (i) {
  return i % 2 === 0 ? 'even' : 'odd';
});

assert.sameValue(Object.getPrototypeOf(obj), null);
assert.sameValue(obj.hasOwnProperty, undefined);
