// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T10
description: >
    Call lastIndexOf(searchString, position) function with object
    arguments
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
}
var __obj2 = {
  valueOf: function() {
    return NaN;
  }
}
var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.lastIndexOf(__obj, __obj2) !== 7) {
  throw new Test262Error('#1: var x; var __obj = {toString:function(){return "\u0041B";}}; var __obj2 = {valueOf:function(){return NaN;}}; var __str = "ABB\u0041BABAB";lastIndexOf(__obj, __obj2) === 7. Actual: ' + __str.lastIndexOf(__obj, __obj2));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
