// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property is created with name length with property
    attributes { DontEnum } and no others
es5id: 10.6_A5_T4
description: Overriding arguments.length property
---*/

var str = "something different";
//CHECK#1
function f1(){
  arguments.length = str;
  return arguments;
}

try{
  if(f1().length !== str){
    throw new Test262Error("#1: A property length have attribute { ReadOnly }");
  }
}
catch(e){
  throw new Test262Error("#1: arguments object don't exists");
}

//CHECK#2
var f2 = function(){
    arguments.length = str;
    return arguments;
  };
try{
  if(f2().length !== str){
    throw new Test262Error("#2: A property length have attribute { ReadOnly }");
  }
}
catch(e){
  throw new Test262Error("#2: arguments object don't exists");
}
