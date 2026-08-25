// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    FunctionExpression within a "do-while" statement is allowed, but no
    function with the given name will appear in the global context
es5id: 12.6.1_A10
description: Also this a test on FunctionExpression
---*/

var check = 0;
do {    
  if(typeof(f) === "function"){
    check = -1;        
    break; 
  } else {
    check = 1;        
    break; 
  }
} while(function f(){});

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (check !== 1) {
	throw new Test262Error('#1: FunctionExpression within a "do-while" statement is allowed, but no function with the given name will appear in the global context');
}
//
//////////////////////////////////////////////////////////////////////////////
