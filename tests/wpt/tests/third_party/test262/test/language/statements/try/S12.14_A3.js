// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Catching system exception with "try" statement
es5id: 12.14_A3
description: Checking if execution of "catch" catches system exceptions
---*/

// CHECK#1
try{
  y;
  throw new Test262Error('#1: "y" lead to throwing exception');
}
catch(e){}

// CHECK#2
var c2=0;
try{
  try{
    someValue;
    throw new Test262Error('#3.1: "someValues" lead to throwing exception');
  }
  finally{
    c2=1;
  }
}
catch(e){
  if (c2!==1){
    throw new Test262Error('#3.2: "finally" block must be evaluated');
  }
}

// CHECK#3
var c3=0,x3=0;
try{
  x3=someValue;
  throw new Test262Error('#3.1: "x3=someValues" lead to throwing exception');
}
catch(err){  	
  x3=1;
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
