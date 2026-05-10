// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Operator x <= y returns ToString(x) <= ToString(y), if Type(Primitive(x))
    is String and Type(Primitive(y)) is String
es5id: 11.8.3_A3.2_T1.1
description: >
    Type(Primitive(x)) and Type(Primitive(y)) vary between primitive
    string and String object
---*/

//CHECK#1
if ("1" <= "1" !== true) {
  throw new Test262Error('#1: "1" <= "1" === true');
}

//CHECK#2
if (new String("1") <= "1" !== true) {
  throw new Test262Error('#2: new String("1") <= "1" === true');
}

//CHECK#3
if ("1" <= new String("1") !== true) {
  throw new Test262Error('#3: "1" <= new String("1") === true');
}

//CHECK#4
if (new String("1") <= new String("1") !== true) {
  throw new Test262Error('#4: new String("1") <= new String("1") === true');
}

//CHECK#5
if ("x" <= "1" !== false) {
  throw new Test262Error('#5: "x" <= "1" === false');
}

//CHECK#6
if ("1" <= "x" !== true) {
  throw new Test262Error('#6: "1" <= "x" === true');
}
