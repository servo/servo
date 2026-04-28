// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    ToObject conversion from Object: The result is the input
    argument (no conversion)
es5id: 9.9_A6
description: Converting from Objects to Object
---*/

function MyObject(val) {
  this.value = val;
  this.valueOf = function() {
    return this.value;
  }
}

var x = new MyObject(1);
var y = Object(x);

assert.sameValue(y.valueOf(), x.valueOf(), 'y.valueOf() must return the same value returned by x.valueOf()');
assert.sameValue(typeof y, typeof x, 'The value of `typeof y` is expected to be typeof x');

assert.sameValue(
  y.constructor.prototype,
  x.constructor.prototype,
  'The value of y.constructor.prototype is expected to equal the value of x.constructor.prototype'
);

assert.sameValue(y, x, 'The value of y is expected to equal the value of x');
