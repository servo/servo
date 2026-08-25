// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property is created with name length with property
    attributes { DontEnum } and no others
es5id: 10.6_A5_T1
description: Checking existence of arguments.length property
---*/

//CHECK#1
function f1(){
  return arguments.hasOwnProperty("length");
}
try{
  if(f1() !== true){
    throw new Test262Error("#1: arguments object doesn't contains property 'length'");
  }
}
catch(e){
  throw new Test262Error("#1: arguments object doesn't exists");
}

//CHECK#2
var f2 = function(){return arguments.hasOwnProperty("length");};
try{
  if(f2() !== true){
    throw new Test262Error("#2: arguments object doesn't contains property 'length'");
  }
}
catch(e){
  throw new Test262Error("#2: arguments object doesn't exists");
}
