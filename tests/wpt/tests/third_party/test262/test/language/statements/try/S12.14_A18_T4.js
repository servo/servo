// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Catching objects with try/catch/finally statement
es5id: 12.14_A18_T4
description: Catching string
---*/

// CHECK#1
try{
  throw "exception #1";
}
catch(e){
  if (e!=="exception #1") throw new Test262Error('#1: Exception ==="exception #1". Actual:  Exception ==='+ e  );
}

// CHECK#2
try{
  throw "exception"+" #1";
}
catch(e){
  if (e!=="exception #1") throw new Test262Error('#2: Exception ==="exception #1". Actual:  Exception ==='+ e  );
}

// CHECK#3
var b="exception #1";
try{
  throw b;
}
catch(e){
  if (e!=="exception #1") throw new Test262Error('#3: Exception ==="exception #1". Actual:  Exception ==='+ e  );
}

// CHECK#4
var a="exception";
var b=" #1";
try{
  throw a+b;
}
catch(e){
  if (e!=="exception #1") throw new Test262Error('#4: Exception ==="exception #1". Actual:  Exception ==='+ e  );
}
