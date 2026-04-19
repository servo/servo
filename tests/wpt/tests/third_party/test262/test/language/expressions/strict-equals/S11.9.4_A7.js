// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are Object-s.
    Return true, if x and y are references to the same Object; otherwise, return false
es5id: 11.9.4_A7
description: >
    Checking Boolean object, Number object, String object, Object
    object
---*/

//CHECK#1
if (new Object() === new Object()) {
  throw new Test262Error('#1: new Object() !== new Object()');
}

//CHECK#2
if (new Object(true) === new Object(true)) {
  throw new Test262Error('#2: new Object() !== new Object()');
}

//CHECK#3
if (new Object(false) === new Object(false)) {
  throw new Test262Error('#3: new Object() !== new Object()');
}

//CHECK#4
if (new Object(+0) === new Object(-0)) {
  throw new Test262Error('#4: new Object(+0) !== new Object(-0)');
}

//CHECK#5
var x, y;
x = {}; 
y = x;
if (!(x === y)) {
  throw new Test262Error('#5: x = {}; y = x; x === y');
}

//CHECK#6
if (new Boolean(true) === new Number(1)) {
  throw new Test262Error('#6 new Boolean(true) === new Number(1)');
}

//CHECK#7
if (new Number(1) === new String("1")) {
  throw new Test262Error('#7: new Number(1) === new String("1")');
}

//CHECK#8
if (new String("1") === new Boolean(true)) {
  throw new Test262Error('#8: new String("x") === new Boolean(true)');
}
