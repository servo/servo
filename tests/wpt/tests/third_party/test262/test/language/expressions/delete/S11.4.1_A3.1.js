// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the property has the DontDelete attribute, return false
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking declared variable
flags: [noStrict]
---*/

//CHECK#1
var x = 1;
if (delete x !== false) {
  throw new Test262Error('#1: var x = 1; delete x === false');
}

//CHECK#2
var y = 1;
if (delete this.y !== false) {
  throw new Test262Error('#2: var y = 1; delete this.y === false');
}

//CHECK#3
function MyFunction() {}
if (delete MyFunction !== false) {
  throw new Test262Error('#3: function MyFunction(){}; delete MyFunction === false');
}

//CHECK#4
var MyObject = new MyFunction();
if (delete MyObject !== false) {
  throw new Test262Error(
    '#4: function MyFunction(){}; var MyObject = new MyFunction(); delete MyObject === false'
  );
}
