// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: FunctionExpression containing "with" statement is admitted
es5id: 13.2.2_A17_T2
description: Throwing an exception within "with" statement
flags: [noStrict]
---*/

this.p1="alert";

__obj={p1:1,getRight:function(){return "right";}};

getRight=function(){return "napravo";};

try {
	(function(){
        with(__obj){
            p1="w1";
            getRight=function(){return false;}
            throw p1;
        }
    })();
} catch (e) {
	resukt = p1;
}


//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (p1!=="alert") {
	throw new Test262Error('#1: p1 === "alert". Actual: p1==='+p1);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (getRight()!=="napravo") {
	throw new Test262Error('#2: getRight() === "napravo". Actual: getRight() === '+getRight());
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__obj.p1!=="w1") {
	throw new Test262Error('#3: __obj.p1 === "w1". Actual: __obj.p1 ==='+__obj.p1);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (__obj.getRight()!==false) {
	throw new Test262Error('#4: __obj.getRight() === false. Actual: __obj.getRight() === '+__obj.getRight());
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#5
if (resukt !== "alert") {
	throw new Test262Error('#5: resukt === "alert". Actual: resukt ==='+resukt);
}
//
//////////////////////////////////////////////////////////////////////////////

var resukt;
