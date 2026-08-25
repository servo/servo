// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: match returns array as specified in 15.10.6.2
es5id: 15.5.4.10_A2_T8
description: >
    Regular expression is /([\d]{5})([-\ ]?[\d]{4})?$/. Last match is
    undefined.  And regular expression object have property lastIndex
    = 0
---*/

var __matches = ["02134", "02134", undefined];

var __re = /([\d]{5})([-\ ]?[\d]{4})?$/;
__re.lastIndex = 0;

var __string = "Boston, MA 02134";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.match(__re).length !== 3) {
  throw new Test262Error('#1: __string = "Boston, MA 02134"; __re = /([\d]{5})([-\ ]?[\d]{4})?$/; __string.match(__re).length=== 3. Actual: ' + __string.match(__re).length);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__string.match(__re).index !== __string.lastIndexOf("0")) {
  throw new Test262Error('#2: __string = "Boston, MA 02134"; __re = /([\d]{5})([-\ ]?[\d]{4})?$/; __re.lastIndex = 0; __string.match(__re).index ===__string.lastIndexOf("0"). Actual: ' + __string.match(__re).index);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
for (var mi = 0; mi < __matches.length; mi++) {
  if (__string.match(__re)[mi] !== __matches[mi]) {
    throw new Test262Error('#3.' + mi + ': __string = "Boston, MA 02134"; __re = /([\d]{5})([-\ ]?[\d]{4})?$/; __matches=["02134", "02134", undefined]; __string.match(__re)[' + mi + ']===__matches[' + mi + ']. Actual: ' + __string.match(__re)[mi]);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
