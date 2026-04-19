// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If two or more formal parameters share the same name, hence
    the same property, the corresponding property is given the value that was
    supplied for the last parameter with this name
es5id: 10.2.1_A2
description: >
    Creating functions initialized with two or more formal parameters,
    which have the same name
flags: [noStrict]
---*/

//CHECK#1
function f1(x, x) {
  return x;
}
if(!(f1(1, 2) === 2)) {
  throw new Test262Error("#1: f1(1, 2) === 2");
}

//CHECK#2
function f2(x, x, x){
  return x*x*x;
}
if(!(f2(1, 2, 3) === 27)){
  throw new Test262Error("f2(1, 2, 3) === 27");
}

//CHECK#3
function f3(x, x) {
  return 'a' + x;
}
if(!(f3(1, 2) === 'a2')){
  throw new Test262Error("#3: f3(1, 2) === 'a2'");
}
