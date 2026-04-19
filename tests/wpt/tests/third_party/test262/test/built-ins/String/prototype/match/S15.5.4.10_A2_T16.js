// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: match returns array as specified in 15.10.6.2
es5id: 15.5.4.10_A2_T16
description: >
    Regular expression is /([\d]{5})([-\ ]?[\d]{4})?$/.  And regular
    expression object have property lastIndex =
    tested_string.lastIndexOf("0")+1
---*/

var __string = "Boston, MA 02134";

var __matches = ["02134"];

var __re = /([\d]{5})([-\ ]?[\d]{4})?$/g;

__re.lastIndex = __string.lastIndexOf("0") + 1;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.match(__re).length !== __matches.length) {
  throw new Test262Error('#1: __string.match(__re).length=== __matches.length. Actual: ' + __string.match(__re).length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__string.match(__re)[0] !== __matches[0]) {
  throw new Test262Error('#3: __string.match(__re)[0]===__matches[0]. Actual: ' + __string.match(__re)[0]);
}
//
//////////////////////////////////////////////////////////////////////////////
