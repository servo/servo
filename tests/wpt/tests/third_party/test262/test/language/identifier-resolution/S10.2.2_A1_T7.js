// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Every execution context has associated with it a scope chain.
    A scope chain is a list of objects that are searched when evaluating an
    Identifier
es5id: 10.2.2_A1_T7
description: Checking scope chain containing function declarations and "with"
flags: [noStrict]
---*/

var x = 0;

var myObj = {x : "obj"};

function f1(){
  function f2(){
    with(myObj){
      return x;
    }
  };
  return f2();

  var x = 1;
}

if(!(f1() === "obj")){
  throw new Test262Error("#1: Scope chain disturbed");
}
