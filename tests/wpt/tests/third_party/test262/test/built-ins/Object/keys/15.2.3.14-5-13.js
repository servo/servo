// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-13
description: >
    Object.keys - own enumerable indexed data property of sparse array
    'O' is defined in returned array
---*/

var obj = [1, , 3, , 5];

Object.defineProperty(obj, 5, {
  value: 7,
  enumerable: false,
  configurable: true
});

Object.defineProperty(obj, 10000, {
  value: "ElementWithLargeIndex",
  enumerable: true,
  configurable: true
});

var arr = Object.keys(obj);

var index;
var initValue = 0;
for (index = 0; index < 3; index++) {
  assert.sameValue(arr[index], initValue.toString(), 'Unexpected property at index: ' + index);
  initValue += 2;
}

assert.sameValue(arr.length, 4, 'arr.length');
assert.sameValue(arr[3], "10000", 'arr[3]');
