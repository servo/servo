// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    FunctionExpression within a "while" IterationStatement is allowed, but no
    function with the given name will appear in the global context
es5id: 12.6.2_A10
description: Testing FunctionExpression too
---*/

var check=0;
while(function f(){}){    
  if(typeof(f) === "function") {
    check = -1;
    break; 
  } else {
    check = 1;
    break; 
  }
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (check !== 1) {
	throw new Test262Error('#1: FunctionExpression inside while construction expression allowed but function not declare');
}
//
//////////////////////////////////////////////////////////////////////////////
