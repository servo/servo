// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Assume F is a Function object. When the [[HasInstance]] method of F is
    called with value V, the following steps are taken: i) If V is not an
    object, return false
es5id: 15.3.5.3_A1_T7
description: V is undefined
---*/

var FACTORY;
FACTORY = Function("name","this.name=name;");

//CHECK#1
if ((undefined instanceof  FACTORY)!==false) {
  throw new Test262Error('#1: Assume F is a Function object. When the [[HasInstance]] method of F is called with value V, the following steps are taken: i) If V is not an object, return false');
}
