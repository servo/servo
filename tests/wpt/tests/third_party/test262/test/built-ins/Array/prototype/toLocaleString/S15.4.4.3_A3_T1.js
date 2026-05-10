// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tolocalestring
info: "[[Get]] from not an inherited property"
es5id: 15.4.4.3_A3_T1
description: "[[Prototype]] of Array instance is Array.prototype"
---*/

var n = 0;
var obj = {
  toLocaleString: function() {
    n++
  }
};
Array.prototype[1] = obj;
var x = [obj];
x.length = 2;
x.toLocaleString();
if (n !== 2) {
  throw new Test262Error('#1: var n = 0; var obj = {toLocaleString: function() {n++}}; Array.prototype[1] = obj; x = [obj]; x.length = 2; x.toLocaleString(); n === 2. Actual: ' + (n));
}
