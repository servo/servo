// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Calling a function as a constructor is inadmissible as long as
    this.any_Function is declared by eval and called
es5id: 13.2.2_A14
description: >
    Calling a function as a constructor after it has been declared by
    eval
flags: [noStrict]
---*/

function FACTORY(){
   this.id = 0;

   eval("function func(){return \"id_string\";}");

   this.id = func();

}
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	var obj = new FACTORY();
} catch (e) {
	throw new Test262Error('#1: var obj = new FACTORY() does not lead to throwing exception');
}
//
//////////////////////////////////////////////////////////////////////////////
