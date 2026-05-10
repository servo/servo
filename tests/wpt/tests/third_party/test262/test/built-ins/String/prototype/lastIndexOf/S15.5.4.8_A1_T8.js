// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T8
description: >
    Call lastIndexOf(searchString, position) function with void 0
    argument of string object
---*/

var __obj = {
  toString: function() {}
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(void 0) evaluates to "undefined" lastIndexOf(void 0) evaluates to lastIndexOf("undefined",0)
if (String(__obj).lastIndexOf(void 0) !== 0) {
  throw new Test262Error('#1: __obj = {toString:function(){}}; String(__obj).lastIndexOf(void 0) === 0. Actual: ' + String(__obj).lastIndexOf(void 0));
}
//
//////////////////////////////////////////////////////////////////////////////
