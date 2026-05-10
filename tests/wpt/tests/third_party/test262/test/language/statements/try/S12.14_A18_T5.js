// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Catching objects with try/catch/finally statement
es5id: 12.14_A18_T5
description: Catching Number
---*/

// CHECK#1
try{
  throw 13;
}
catch(e){
  if (e!==13) throw new Test262Error('#1: Exception ===13. Actual:  Exception ==='+ e  );
}

// CHECK#2
try{
  throw 10+3;
}
catch(e){
  if (e!==13) throw new Test262Error('#2: Exception ===13. Actual:  Exception ==='+ e  );
}

// CHECK#3
var b=13;
try{
  throw b;
}
catch(e){
  if (e!==13) throw new Test262Error('#3: Exception ===13. Actual:  Exception ==='+ e  );
}

// CHECK#4
var a=3;
var b=10;
try{
  throw a+b;
}
catch(e){
  if (e!==13) throw new Test262Error('#4: Exception ===13. Actual:  Exception ==='+ e  );
}

// CHECK#5
try{
  throw 2.13;
}
catch(e){
  if (e!==2.13) throw new Test262Error('#5: Exception ===2.13. Actual:  Exception ==='+ e  );
}

// CHECK#6
var ex=2/3;
try{
  throw 2/3;
}
catch(e){
  if (e!==ex) throw new Test262Error('#6: Exception ===2/3. Actual:  Exception ==='+ e  );
}

// CHECK#7
try{
  throw NaN;
}
catch(e){
  assert.sameValue(e, NaN, "e is NaN");
}

// CHECK#8
try{
  throw +Infinity;
}
catch(e){
  if (e!==+Infinity) throw new Test262Error('#8: Exception ===+Infinity. Actual:  Exception ==='+ e  );
}

// CHECK#9
try{
  throw -Infinity;
}
catch(e){
  if (e!==-Infinity) throw new Test262Error('#9: Exception ===-Infinity. Actual:  Exception ==='+ e  );
}

// CHECK#10
try{
  throw +0;
}
catch(e){
  if (e!==+0) throw new Test262Error('#10: Exception ===+0. Actual:  Exception ==='+ e  );
}

// CHECK#11
try{
  throw -0;
}
catch(e){
  assert.sameValue(e, -0);
}
