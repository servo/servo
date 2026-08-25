// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Variable instantiation is performed using the global object as
    the variable object and using property attributes { DontDelete }
es5id: 10.4.1_A1_T2
description: Checking if deleting variable x, that is defined as x = 1, fails
flags: [noStrict]
---*/

x = 1;

if (this.x !== 1) {
  throw new Test262Error("#1: variable x is a property of global object");
}

if(delete this.x !== true){
  throw new Test262Error("#2: variable x has property attribute DontDelete");
}
