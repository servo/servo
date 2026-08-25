// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Return Statement for automatic semicolon insertion
es5id: 7.9_A3
description: Try use return \n Expression construction
---*/

//CHECK#1
function f1()
{
  return 1;
}
if (f1() !== 1) {
  throw new Test262Error('#1: Check return statement for automatic semicolon insertion');
}

//CHECK#2
function f2()
{
  return
  1;
}
if (f2() !== undefined) {
  throw new Test262Error('#2: Check return statement for automatic semicolon insertion');
}
