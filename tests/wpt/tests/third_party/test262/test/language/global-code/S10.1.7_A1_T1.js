// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The this value associated with an executioncontext is immutable
es5id: 10.1.7_A1_T1
description: Checking if deleting "this" fails
---*/

//CHECK#1
if (delete this !== true)
  throw new Test262Error('#1: The this value associated with an executioncontext is immutable. Actual: this was deleted');
