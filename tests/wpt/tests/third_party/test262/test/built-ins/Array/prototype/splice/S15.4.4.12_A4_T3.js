// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Get]] from not an inherited property"
esid: sec-array.prototype.splice
description: >
    [[Prototype]] of Array instance is Array.prototype, [[Prototype]
    of Array.prototype is Object.prototype
---*/

Array.prototype[0] = -1;
var x = [];
x.length = 1;
var arr = x.splice(0, 1);

if (arr.length !== 1) {
  throw new Test262Error('#1: Array.prototype[0] = -1; x = []; x.length = 1; var arr = x.splice(0,1); arr.length === 1. Actual: ' + (arr.length));
}

if (arr[0] !== -1) {
  throw new Test262Error('#2: Array.prototype[0] = -1; x = []; x.length = 1; var arr = x.splice(0,1); arr[0] === -1. Actual: ' + (arr[0]));
}

delete arr[0];

if (arr[0] !== -1) {
  throw new Test262Error('#3: Array.prototype[0] = -1; x = []; x.length = 1; var arr = x.splice(0,1); delete arr[0]; arr[0] === -1. Actual: ' + (arr[0]));
}

if (x.length !== 0) {
  throw new Test262Error('#4: Array.prototype[0] = -1; x = []; x.length = 1; var arr = x.splice(0,1); x.length === 0. Actual: ' + (x.length));
}

if (x[0] !== -1) {
  throw new Test262Error('#5: Array.prototype[0] = -1; x = []; x.length = 1; var arr = x.splice(0,1); x[0] === -1. Actual: ' + (x[0]));
}

Object.prototype[0] = -1;
Object.prototype.length = 1;
Object.prototype.splice = Array.prototype.splice;
x = {};
var arr = x.splice(0, 1);

if (arr.length !== 1) {
  throw new Test262Error('#6: Object.prototype[0] = -1; Object.prototype.length = 1; Object.prototype.splice = Array.prototype.splice; x = {}; var arr = x.splice(0,1); arr.length === 1. Actual: ' + (arr.length));
}

if (arr[0] !== -1) {
  throw new Test262Error('#7: Object.prototype[0] = -1; Object.prototype.length = 1; Object.prototype.splice = Array.prototype.splice; x = {}; var arr = x.splice(0,1); arr[0] === -1. Actual: ' + (arr[0]));
}

delete arr[0];

if (arr[0] !== -1) {
  throw new Test262Error('#8: Object.prototype[0] = -1; Object.prototype.length = 1; Object.prototype.splice = Array.prototype.splice; x = {}; var arr = x.splice(0,1); delete arr[0]; arr[0] === -1. Actual: ' + (arr[0]));
}

if (x.length !== 0) {
  throw new Test262Error('#9: Object.prototype[0] = -1; Object.prototype.length = 1; Object.prototype.splice = Array.prototype.splice; x = {}; var arr = x.splice(0,1); x.length === 0. Actual: ' + (x.length));
}

if (x[0] !== -1) {
  throw new Test262Error('#10: Object.prototype[0] = -1; Object.prototype.length = 1; Object.prototype.splice = Array.prototype.splice; x = {}; var arr = x.splice(0,1); x[0] === -1. Actual: ' + (x[0]));
}
