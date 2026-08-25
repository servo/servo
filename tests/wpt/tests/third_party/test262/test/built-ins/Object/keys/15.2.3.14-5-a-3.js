// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-5-a-3
description: >
    Object.keys - 'enumerable' attribute of element of returned array
    is correct
---*/

var obj = {
  prop1: 100
};

var array = Object.keys(obj);
var desc = Object.getOwnPropertyDescriptor(array, "0");
var result = false;
for (var index in array) {
  if (obj.hasOwnProperty(array[index]) && array[index] === "prop1") {
    result = true;
  }
}

assert(result, 'result !== true');
assert(desc.hasOwnProperty("enumerable"), 'desc.hasOwnProperty("enumerable") !== true');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
