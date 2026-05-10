// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String instance has not [[call]] property
es5id: 15.5.5_A1_T2
description: Checking if creating new "String("a|b")()" fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  String("a|b")();
  throw new Test262Error('#1: String("a|b")() lead to throwing exception');
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error('#1.1: Exception is instance of TypeError. Actual: exception is ' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
