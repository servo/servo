// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Operator x >= y returns ToString(x) >= ToString(y), if Type(Primitive(x))
    is String and Type(Primitive(y)) is String
es5id: 11.8.4_A3.2_T1.2
description: >
    Type(Primitive(x)) and Type(Primitive(y)) vary between Object
    object and Function object
---*/

//CHECK#1
if (({} >= function(){return 1}) !== ({}.toString() >= function(){return 1}.toString())) {
  throw new Test262Error('#1: ({} >= function(){return 1}) === ({}.toString() >= function(){return 1}.toString())');
}

//CHECK#2
if ((function(){return 1} >= {}) !== (function(){return 1}.toString() >= {}.toString())) {
  throw new Test262Error('#2: (function(){return 1} >= {}) === (function(){return 1}.toString() >= {}.toString())');
}

//CHECK#3
if ((function(){return 1} >= function(){return 1}) !== (function(){return 1}.toString() >= function(){return 1}.toString())) {
  throw new Test262Error('#3: (function(){return 1} >= function(){return 1}) === (function(){return 1}.toString() >= function(){return 1}.toString())');
}

//CHECK#4
if (({} >= {}) !== ({}.toString() >= {}.toString())) {
  throw new Test262Error('#4: ({} >= {}) === ({}.toString() >= {}.toString())');
}
