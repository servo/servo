// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Assume F is a Function object. When the [[HasInstance]] method of F is called with value V and V is an object, the following steps are taken:
    i) Call the [[Get]] method of F with property name "prototype".
    ii) Let O be Result(i).
    iii) O is not an object, throw a TypeError exception
es5id: 15.3.5.3_A2_T2
description: F.prototype is undefined, and V is empty object
---*/

var FACTORY;
FACTORY = new Function;

FACTORY.prototype = undefined;

var obj;
obj={};

//CHECK#1
try {
  obj instanceof  FACTORY;
  throw new Test262Error('#1: O is not an object, throw a TypeError exception');
} catch (e) {
  if (!(e instanceof TypeError)) {
  	throw new Test262Error('#1.1: O is not an object, throw a TypeError exception');
  }
}
