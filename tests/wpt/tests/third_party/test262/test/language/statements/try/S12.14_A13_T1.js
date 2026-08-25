// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Using "try" with "catch" or "finally" statement with a "return" statement
es5id: 12.14_A13_T1
description: Using try/catch syntax construction
---*/

// CHECK#1
function myFunction1(){
  try{
    return 1;
  }
  catch(err){
  	throw new Test262Error('#1.1: "return 1" inside function does not lead to throwing exception');
    return 0;
  }
  return 2;
}
var x1=myFunction1();
if(x1!==1){
  throw new Test262Error('#1.2: x1===1. Actual: x1==='+x1);
}

// CHECK#2
function myFunction2(){
  try{
    throw "exc";
    return 1;
  }catch(err){  	
    return 2;
  }
  return 3;
}
var x2=myFunction2();
if (x2!==2){
  throw new Test262Error('#2: x2===2. Actual: x2==='+x2);
}

// CHECK#3
function myFunction3(){
  try{
    return someValue;
  }catch(err){  	
    return 1;
  }
  return 2;
}
var x3=myFunction3();
if (x3!==1){
  throw new Test262Error('#3: x3===1. Actual: x3==='+x3);
}

// CHECK#4
function myFunction4(){
  try{
    throw "ex1";
    return 1;
  }catch(err){
    throw "ex2"
    return 0;
  }
  return 2;
}
try{
  var x4=myFunction4();
  throw new Test262Error('#4.1: Throwing exception inside function lead to throwing exception outside this function');
}
catch(e){
  if(e==="ex1"){
    throw new Test262Error('#4.2: Exception !=="ex1". Actual: catch previous exception');
  }
  if(e!=="ex2"){
    throw new Test262Error('#4.3: Exception ==="ex2". Actual:  Exception ==='+ e  );
  }
}
