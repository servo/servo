// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-3-1
description: >
    Object.getOwnPropertyNames - elements of the returned array start
    from index 0
---*/

var obj = {
  prop1: 1001
};

var arr = Object.getOwnPropertyNames(obj);

assert(arr.hasOwnProperty(0), 'arr.hasOwnProperty(0) !== true');
assert.sameValue(arr[0], "prop1", 'arr[0]');
