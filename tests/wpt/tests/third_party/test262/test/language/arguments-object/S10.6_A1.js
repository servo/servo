// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When control enters an execution context for function code,
    an arguments object is created and initialised
es5id: 10.6_A1
description: Executing function which uses arguments object
---*/

//CHECK#1
function f1(){
  return arguments;
}

try{
  var x = f1();
}
catch(e){
  throw new Test262Error("#1: arguments doesn't exists");
}

//CHECK#2
var f2 = function(){
  return arguments;
}

try{
  var x = f2();
}
catch(e){
  throw new Test262Error("#2: arguments doesn't exists");
}
