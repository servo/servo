// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Using arguments object within a "with" Expression that is nested in a
    function is admitted
es5id: 13.2.2_A18_T1
description: "Object is declared with \"var __obj={callee:\"a\"}\""
flags: [noStrict]
---*/

var callee=0, b;

var __obj={callee:"a"};

result=(function(){
    with (arguments){
        callee=1;
        b=true;
    }
    return arguments;
})(__obj);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (callee !== 0) {
	throw new Test262Error('#1: callee === 0. Actual: callee ==='+callee);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj.callee !== "a") {
	throw new Test262Error('#2: __obj.callee === "a". Actual: __obj.callee==='+__obj.callee);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (result.callee !== 1) {
	throw new Test262Error('#3: result.callee === 1. Actual: result.callee ==='+result.callee);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (!(this.b)) {
	throw new Test262Error('#4: this.b === true. Actual: this.b ==='+this.b);
}
//
//////////////////////////////////////////////////////////////////////////////
