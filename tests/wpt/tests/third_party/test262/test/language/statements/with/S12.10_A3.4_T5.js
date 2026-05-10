// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    No matter how control leaves the embedded 'Statement',
    the scope chain is always restored to its former state
es5id: 12.10_A3.4_T5
description: >
    Using "with" statement within iteration statement, leading to
    completion by break
flags: [noStrict]
---*/

this.p1 = 1;

var result = "result";

var myObj = {
    p1: 'a', 
    value: 'myObj_value',
    valueOf : function(){return 'obj_valueOf';}
}

do {
    with(myObj){
        break;
        p1 = 'x1';
    }
} while(false);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if(p1 !== 1){
  throw new Test262Error('#1: p1 === 1. Actual:  p1 ==='+ p1  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if(myObj.p1 !== "a"){
  throw new Test262Error('#2: myObj.p1 === "a". Actual:  myObj.p1 ==='+ myObj.p1  );
}
//
//////////////////////////////////////////////////////////////////////////////
