// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: match returns array as specified in 15.10.6.2
es5id: 15.5.4.10_A2_T5
description: Regular expression is /\D{2}/g
---*/

var __matches = ["ab", "cd"];

var __string = "123456abcde7890";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.match(/\D{2}/g).length !== 2) {
  throw new Test262Error('#1: __string = "123456abcde7890"; __string.match(/\\D{2}/g).length=== 2. Actual: ' + __string.match(/\D{2}/g).length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
for (var mi = 0; mi < __matches.length; mi++) {
  if (__string.match(/\D{2}/g)[mi] !== __matches[mi]) {
    throw new Test262Error('#2.' + mi + ': __matches=["ab", "cd"]; __string = "123456abcde7890"; __string.match(/\\D{2}/g)[' + mi + ']===__matches[' + mi + ']. Actual: ' + __string.match(/\D{2}/g)[mi]);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
