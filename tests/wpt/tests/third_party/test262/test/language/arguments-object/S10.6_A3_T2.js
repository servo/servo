// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property is created with name callee with property
    attributes { DontEnum } and no others
es5id: 10.6_A3_T2
description: Checking if enumerating the arguments.callee property fails
---*/

//CHECK#1
function f1(){
  for(var x in arguments){
    if (x === "callee"){
      return false;
    }
  }
  return true;
}

try{
  if(!f1()){
    throw new Test262Error("#1: A property callee don't have attribute { DontEnum }");
  }
}
catch(e){
  throw new Test262Error("#1: arguments object don't exists");
}

//CHECK#2
var f2 = function(){
  for(var x in arguments){
    if (x === "callee"){
      return false;
    }
  }
  return true;
}

try{
  if(!f2()){
    throw new Test262Error("#2: A property callee don't have attribute { DontEnum }");
  }
}
catch(e){
  throw new Test262Error("#2: arguments object don't exists");
}
