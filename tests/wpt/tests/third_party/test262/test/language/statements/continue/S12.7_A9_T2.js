// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Continue inside of try-catch nested in a loop is allowed
es5id: 12.7_A9_T2
description: Using "continue" within catch Block that is within a loop
---*/

var x=0,y=0;

(function(){
FOR : for(;;){
	try{
		x++;
		if(x===10)return;
		throw 1;
	} catch(e){
		continue;
	}	
}
})();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (x!==10) {
	throw new Test262Error('#1: Continue inside of try-catch nested in loop is allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
