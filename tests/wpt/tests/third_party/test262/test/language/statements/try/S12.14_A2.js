// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Throwing exception with "throw" and catching it with "try" statement
es5id: 12.14_A2
description: >
    Checking if execution of "catch" catches an exception thrown with
    "throw"
---*/

// CHECK#1
try {
  throw "catchme";	
  throw new Test262Error('#1: throw "catchme" lead to throwing exception');
}
catch(e){}

// CHECK#2
var c2=0;
try{
  try{
    throw "exc";
    throw new Test262Error('#2.1: throw "exc" lead to throwing exception');
  }finally{
    c2=1;
  }
}
catch(e){
  if (c2!==1){
    throw new Test262Error('#2.2: "finally" block must be evaluated');
  }
}
 
// CHECK#3
var c3=0;
try{
  throw "exc";
  throw new Test262Error('#3.1: throw "exc" lead to throwing exception');
}
catch(err){  	
  var x3=1;
}
finally{
  c3=1;
}
if (x3!==1){
  throw new Test262Error('#3.2: "catch" block must be evaluated');
}  
if (c3!==1){
  throw new Test262Error('#3.3: "finally" block must be evaluated');
}
