// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Get]] from not an inherited property"
esid: sec-array.prototype.slice
description: "[[Prototype]] of Array instance is Array.prototype"
---*/

Array.prototype[1] = 1;
var x = [0];
x.length = 2;
var arr = x.slice();

if (arr[0] !== 0) {
  throw new Test262Error('#1: Array.prototype[1] = 1; x = [0]; x.length = 2; var arr = x.slice(); arr[0] === 0. Actual: ' + (arr[0]));
}

if (arr[1] !== 1) {
  throw new Test262Error('#2: Array.prototype[1] = 1; x = [0]; x.length = 2; var arr = x.slice(); arr[1] === 1. Actual: ' + (arr[1]));
}

if (arr.hasOwnProperty('1') !== true) {
  throw new Test262Error('#3: Array.prototype[1] = 1; x = [0]; x.length = 2; var arr = x.slice(); arr.hasOwnProperty(\'1\') === true. Actual: ' + (arr.hasOwnProperty('1')));
}
