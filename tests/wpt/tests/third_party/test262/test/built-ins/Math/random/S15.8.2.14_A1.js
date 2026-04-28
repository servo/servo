// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Math.random() returns a number value with positive sign, greater than or
    equal to 0 but less than 1
es5id: 15.8.2.14_A1
description: >
    Checking if Math.random() is a number between 0 and 1, calling
    Math.random() 100 times
---*/

// CHECK#1
for (var i = 0; i < 100; i++)
{
  var val = Math.random();

  assert.sameValue(
    typeof val, 'number', 'should not produce a non-numeric value: ' + val
  );
  assert.notSameValue(val, NaN, 'should not produce NaN');

  if (val < 0 || val >= 1)
  {
    throw new Test262Error("#1: Math.random() = " + val);
  }
}
