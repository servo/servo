// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.indexOf(searchString, position)
es5id: 15.5.4.7_A1_T10
description: Call indexOf(searchString, position) function with object arguments
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
}
var __obj2 = {
  valueOf: function() {
    return true;
  }
}
var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.indexOf(__obj, __obj2) !== 3) {
  throw new Test262Error('#1: var x; var __obj = {toString:function(){return "\u0041B";}}; var __obj2 = {valueOf:function(){return true;}}; var __str = "ABB\u0041BABAB"; __str.indexOf(__obj, __obj2) === 3. Actual: ' + __str.indexOf(__obj, __obj2));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
