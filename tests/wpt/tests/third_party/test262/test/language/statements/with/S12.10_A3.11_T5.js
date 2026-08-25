// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    No matter how control leaves the embedded 'Statement',
    the scope chain is always restored to its former state
es5id: 12.10_A3.11_T5
description: >
    Calling a function within "with" statement declared without the
    statement, leading to completion by exception
flags: [noStrict]
---*/

this.p1 = 1;
var result = "result";
var value = "value";
var myObj = {p1: 'a', 
             value: 'myObj_value',
             valueOf : function(){return 'obj_valueOf';}
}

try {
  var f = function(){
    throw value;
    p1 = 'x1';
  }
  with(myObj){
    f();
  }
} catch(e){
  result = e;
}

if(!(p1 === 1)){
  throw new Test262Error('#1: p1 === 1. Actual:  p1 ==='+ p1  );
}

if(!(myObj.p1 === "a")){
  throw new Test262Error('#2: myObj.p1 === "a". Actual:  myObj.p1 ==='+ myObj.p1  );
}

if(!(result === "value")){
  throw new Test262Error('#3: result === "value". Actual:  result ==='+ result  );
}
