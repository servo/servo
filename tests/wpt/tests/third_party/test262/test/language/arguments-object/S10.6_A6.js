// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The initial value of the created property length is the number
    of actual parameter values supplied by the caller
es5id: 10.6_A6
description: Create function, that returned arguments.length
---*/

function f1(){
  return arguments.length;
}

//CHECK#1
if(!(f1() === 0)){
  throw new Test262Error('#1: argument.length === 0');
}

//CHECK#2
if(!(f1(0) === 1)){
  throw new Test262Error('#2: argument.length === 1');
}

//CHECK#3
if(!(f1(0, 1) === 2)){
  throw new Test262Error('#3: argument.length === 2');
}

//CHECK#4
if(!(f1(0, 1, 2) === 3)){
  throw new Test262Error('#4: argument.length === 3');
}

//CHECK#5
if(!(f1(0, 1, 2, 3) === 4)){
  throw new Test262Error('#5: argument.length === 4');
}

var f2 = function(){return arguments.length;};

//CHECK#6
if(!(f2() === 0)){
  throw new Test262Error('#6: argument.length === 0');
}

//CHECK#7
if(!(f2(0) === 1)){
  throw new Test262Error('#7: argument.length === 1');
}

//CHECK#8
if(!(f2(0, 1) === 2)){
  throw new Test262Error('#8: argument.length === 2');
}

//CHECK#9
if(!(f2(0, 1, 2) === 3)){
  throw new Test262Error('#9: argument.length === 3');
}

//CHECK#10
if(!(f2(0, 1, 2, 3) === 4)){
  throw new Test262Error('#10: argument.length === 4');
}
