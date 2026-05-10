// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Empty string, 0, false are all equal (==) to each other, since they all
    evaluate to 0
es5id: 8.4_A8
description: Compare empty string with undefined, null, 0 and false
---*/

var str='';

////////////////////////////////////////////////////////////
// CHECK#1
if (str == undefined){
  throw new Test262Error('#1: Empty string and undefined are not equal (!=) to each other');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#2
if (str == null){
  throw new Test262Error('#1: Empty string and Null are not equal (!=) to each other');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#3
if (str != 0){
  throw new Test262Error('#3: Empty string and 0 are equal (==) to each other, since they all evaluate to 0');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#4
if (str != false){
  throw new Test262Error('#4: Empty string and false are equal (==) to each other, since they all evaluate to 0');
}
//
/////////////////////////////////////////////////////////////
