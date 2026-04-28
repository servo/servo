// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: NaN not greater or equal zero
es5id: 8.5_A5
description: Compare NaN with zero
---*/

var x = NaN;
var x_geq_0=(x >= 0.0);
var x_leq_0=(x <= 0.0);
var x_leq_0_OR_geq_0=(x <= 0.0)||(x >= 0.0);
var x_geq_0_ADD_leq_0=(x >= 0.0) + (x <= 0.0);


///////////////////////////////////////////////////
// CHECK#1
if (x_geq_0){
  throw new Test262Error('#1: NaN not greater or equal zero');
}
//
///////////////////////////////////////////////////

///////////////////////////////////////////////////
// CHECK#2
if (x_leq_0){
  throw new Test262Error('#2: NaN not less or equal zero');
}
//
///////////////////////////////////////////////////

///////////////////////////////////////////////////
// CHECK#3
if (x_leq_0_OR_geq_0){
  throw new Test262Error('#3: NaN not less or equal zero OR greater or equal zero');
}
//
///////////////////////////////////////////////////

///////////////////////////////////////////////////
// CHECK#4
if (x_geq_0_ADD_leq_0){
  throw new Test262Error('#4: NaN not less or equal zero ADD greater or equal zero');
}
//
///////////////////////////////////////////////////
