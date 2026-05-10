// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The value of Math.ceil(x) is the same as the value of -Math.floor(-x)
es5id: 15.8.2.6_A7
description: >
    Checking if Math.ceil(x) equals to -Math.floor(-x) on 2000
    floating point argument values
---*/

// CHECK#1
for (var i = -1000; i < 1000; i++)
{
  var x = i / 10.0;
  assert.sameValue(Math.ceil(x), -Math.floor(-x), 'Math.ceil(i / 10.0) must return -Math.floor(-x)');
}
