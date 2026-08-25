// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-6-5
description: >
    Object.keys - the order of elements in returned array is the same
    with the order of properties in 'O' (any other built-in object)
---*/

var obj = new Date(0);
obj.prop1 = 100;
obj.prop2 = "prop2";

var tempArray = [];
for (var p in obj) {
  if (obj.hasOwnProperty(p)) {
    tempArray.push(p);
  }
}

var returnedArray = Object.keys(obj);

for (var index in returnedArray) {
  assert.sameValue(tempArray[index], returnedArray[index], 'tempArray[index]');
}
