// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Infinity is the same as +Infinity
es5id: 8.5_A8
description: Compare Infinity and +Infinity
---*/

var p_inf=+Infinity;
var inf=Infinity;

///////////////////////////////////////////////////////
//
if (p_inf!==inf){
  throw new Test262Error('#1: Infinity is the same as +Infinity');
}
//
//////////////////////////////////////////////////////////
