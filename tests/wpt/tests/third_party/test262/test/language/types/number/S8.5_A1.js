// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: NaN !== NaN
es5id: 8.5_A1
description: Compare NaN with NaN
---*/

var x = Number.NaN;
var x_ = Number.NaN;

///////////////////////////////////////////////////////
// CHECK #1
if (x === x_){
  throw new Test262Error('#1: NaN !== NaN ');
}
//
//////////////////////////////////////////////////////////
