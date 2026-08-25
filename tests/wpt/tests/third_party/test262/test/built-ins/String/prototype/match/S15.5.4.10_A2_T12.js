// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: match returns array as specified in 15.10.6.2
es5id: 15.5.4.10_A2_T12
description: >
    Regular expression is variable that have value /([\d]{5})([-\
    ]?[\d]{4})?$/g
---*/

var __matches = ["02134"];

var __string = "Boston, MA 02134";

var __re = /([\d]{5})([-\ ]?[\d]{4})?$/g;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.match(__re).length !== __matches.length) {
  throw new Test262Error('#1: __string.match(__re).length=== __matches.length. Actual: ' + __string.match(__re).length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__string.match(__re)[0] !== __matches[0]) {
  throw new Test262Error('#2: __string.match(__re)[0]===__matches[0]. Actual: ' + __string.match(__re)[0]);
}
//
//////////////////////////////////////////////////////////////////////////////
