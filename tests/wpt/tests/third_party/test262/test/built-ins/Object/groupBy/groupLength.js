// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Callback can return numbers that are converted to property keys
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    c. If next is false, then
      i. Return groups.
  ...
includes: [compareArray.js]
features: [array-grouping]
---*/

const arr = ['hello', 'test', 'world'];

const obj = Object.groupBy(arr, function (i) { return i.length; });

assert.compareArray(Object.keys(obj), ['4', '5']);
assert.compareArray(obj['5'], ['hello', 'world']);
assert.compareArray(obj['4'], ['test']);
