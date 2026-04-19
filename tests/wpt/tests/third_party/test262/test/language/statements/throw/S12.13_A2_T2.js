// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    "throw Expression" returns (throw, GetValue(Result(1)), empty), where 1
    evaluates Expression
es5id: 12.13_A2_T2
description: Throwing null
---*/

// CHECK#1
try{
  throw null;
}
catch(e){
  if (e!==null) throw new Test262Error('#1: Exception === null. Actual:  Exception ==='+ e  );
}
