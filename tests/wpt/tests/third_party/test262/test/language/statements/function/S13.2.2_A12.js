// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Calling a function as a constructor is possible as long as
    this.any_Function is declared and called
es5id: 13.2.2_A12
description: >
    Calling a function as a constructor after it has been declared
    with "function func()"
---*/

function FACTORY(){
   this.id = 0;
      
   this.id = func();
   
   function func(){
      return "id_string";
   }
     
}
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	var obj = new FACTORY();
} catch (e) {
	throw new Test262Error('#1: var obj = new FACTORY() does not lead to throwing exception. Actual: Exception is '+e);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (obj.id !== "id_string") {
	throw new Test262Error('#2: obj.id === "id_string". Actual: obj.id ==='+obj.id);
}
//
//////////////////////////////////////////////////////////////////////////////
