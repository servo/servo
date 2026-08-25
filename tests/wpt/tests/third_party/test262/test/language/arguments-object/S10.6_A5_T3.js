// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property is created with name length with property
    attributes { DontEnum } and no others
es5id: 10.6_A5_T3
description: Checking if deleting arguments.length property fails
---*/

//CHECK#1
function f1(){
  return (delete arguments.length);
}

try{
  if(!f1()){
    throw new Test262Error("#1: A property length have attribute { DontDelete }");
  }
}
catch(e){
  throw new Test262Error("#1: arguments object don't exists");
}

//CHECK#2
var f2 = function(){
  return (delete arguments.length);
}

try{
  if(!f2()){
    throw new Test262Error("#2: A property length have attribute { DontDelete }");
  }
}
catch(e){
  throw new Test262Error("#2: arguments object don't exists");
}
