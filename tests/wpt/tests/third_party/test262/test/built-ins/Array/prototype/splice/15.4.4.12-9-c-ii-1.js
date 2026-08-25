// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
    Array.prototype.splice will splice an array even when
    Array.prototype has index '0' set to read-only and 'fromPresent'
    less than 'actualDeleteCount (Step 9.c.ii)
---*/

var arr = ["a", "b", "c"];
Array.prototype[0] = "test";
var newArr = arr.splice(2, 1, "d");

var verifyValue = false;
verifyValue = arr.length === 3 && arr[0] === "a" && arr[1] === "b" && arr[2] === "d" &&
  newArr[0] === "c" && newArr.length === 1;

var verifyEnumerable = false;
for (var p in newArr) {
  if (newArr.hasOwnProperty("0") && p === "0") {
    verifyEnumerable = true;
  }
}

var verifyWritable = false;
newArr[0] = 12;
verifyWritable = newArr[0] === 12;

var verifyConfigurable = false;
delete newArr[0];
verifyConfigurable = newArr.hasOwnProperty("0");

assert(verifyValue, 'verifyValue !== true');
assert.sameValue(verifyConfigurable, false, 'verifyConfigurable');
assert(verifyEnumerable, 'verifyEnumerable !== true');
assert(verifyWritable, 'verifyWritable !== true');
