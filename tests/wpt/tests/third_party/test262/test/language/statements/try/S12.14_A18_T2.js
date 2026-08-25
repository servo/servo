// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Catching objects with try/catch/finally statement
es5id: 12.14_A18_T2
description: Catching null
---*/

// CHECK#1
try{
  throw null;
}
catch(e){
  if (e!==null) throw new Test262Error('#1: Exception ===null. Actual: '+e);
}
