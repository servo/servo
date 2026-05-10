// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy populates object with correct keys and values
info: |
  Object.groupBy ( items, callbackfn )
  ...
includes: [compareArray.js]
features: [array-grouping]
---*/

const array = [1, 2, 3];

const obj = Object.groupBy(array, function (i) {
  return i % 2 === 0 ? 'even' : 'odd';
});

assert.compareArray(Object.keys(obj), ['odd', 'even']);
assert.compareArray(obj['even'], [2]);
assert.compareArray(obj['odd'], [1, 3]);
