// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If any value is NaN, the result of Math.min is NaN
es5id: 15.8.2.12_A2
description: >
    The script tests Math.min giving 1, 2 and 3 arguments to the
    function where at least one of the arguments is NaN
---*/

// CHECK#1
assert.sameValue(Math.min(NaN), NaN, "NaN");

// CHECK#2
var vals = new Array();
vals[0] = -Infinity;
vals[1] = -0.000000000000001;
vals[2] = -0;
vals[3] = +0
vals[4] = 0.000000000000001;
vals[5] = +Infinity;
vals[6] = NaN;
var valnum = 7;

var args = new Array();
for (var i = 0; i <= 1; i++)
{
  args[i] = NaN;
  for (var j = 0; j < valnum; j++)
  {
    args[1 - i] = vals[j];
    assert.sameValue(
      Math.min(args[0], args[1]),
      NaN,
      "min(" + args[0] + ", " + args[1] + ")"
    );
  }
}

// CHECK #3
var k = 1;
var l = 2;
for (var i = 0; i <= 2; i++)
{
  args[i] = NaN;
  if (i === 1)
  {
    k = 0;
  } else if (i === 2)
  {
    l = 1;
  }
  for (var j = 0; j < valnum; j++)
  {
    for (var jj = 0; jj < valnum; jj++)
    {
      args[k] = vals[j];
      args[l] = vals[jj];
      assert.sameValue(
        Math.min(args[0], args[1], args[2]),
        NaN,
        "min(" + args[0] + ", " + args[1] + ", " + args[2] + ")"
      );
    }
  }
}
