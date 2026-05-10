// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The production TryStatement: \"try Block Catch Finally\""
es5id: 12.14_A6
description: >
    Executing sequence of "try" statements, using counters with
    varying values within
---*/

// CHECK#1
var c1=0;
try {
  c1+=1;
  y;
  throw new Test262Error('#1.1: "y" lead to throwing exception');
}
catch (e) {
  c1*=2;
}
if (c1!==2){
  throw new Test262Error('#1.2: Sequence evaluation of commands try/catch is 1. try, 2. catch');	
}

// CHECK#2
var c2=0;
try{
  c2+=1;
}
finally{
  c2*=2;
}
if (c2!==2){
  throw new Test262Error('#2: Sequence evaluation of commands try/finally is 1. try, 2. finally');
}

// CHECK#3
var c3=0;
try{
  c3=1;
  z;
}
catch(err){
  c3*=2;
}
finally{
  c3+=1;
}
if (c3!==3){
  throw new Test262Error('#3: Sequence evaluation of commands try/catch/finally(with exception) is 1. try, 2. catch, 3. finally');
}	

// CHECK#4
var c4=0;
try{
  c4=1;
}
catch(err){
  c4*=3;
}
finally{
  c4+=1;
}
if (c4!==2){
  throw new Test262Error('#4: Sequence evaluation of commands try/catch/finally(without exception) is 1. try, 2. finally');
}
