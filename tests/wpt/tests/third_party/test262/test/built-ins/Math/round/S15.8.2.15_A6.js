// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If x is equal to 0 or greater than 0, or if x is less than -0.5,
    Math.round(x) is equal to Math.floor(x+0.5)
es5id: 15.8.2.15_A6
description: >
    Checking if Math.round(x) is equal to Math.floor(x+0.5), where x
    equals to 0, greater than 0, or is less than -0.5; this check is
    performed on 2000 argument x values
---*/

// CHECK#1
for (var i = 0; i <= 1000; i++)
{
  var x = i / 10.0;

  assert.sameValue(
    Math.round(x),
    Math.floor(x + 0.5),
    'Math.round(i / 10.0) must return the same value returned by Math.floor(x + 0.5)'
  );
}

for (i = -5; i >= -1000; i--)
{
  if (i === -5)
  {
    x = -0.500000000000001;
  } else
  {
    x = i / 10.0;
  }

  assert.sameValue(
    Math.round(x),
    Math.floor(x + 0.5),
    'Math.round(i / 10.0) must return the same value returned by Math.floor(x + 0.5)'
  );
}
