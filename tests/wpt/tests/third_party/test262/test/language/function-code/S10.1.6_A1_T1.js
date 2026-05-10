// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The activation object is initialised with a property with name arguments
    and attributes {DontDelete}
es5id: 10.1.6_A1_T1
description: Checking if deleting function parameter is possible
flags: [noStrict]
---*/

//CHECK#1
function f1(a){
  delete a;
  return a;
}
if (f1(1) !== 1)
  throw new Test262Error('#1: Function parameter was deleted');
