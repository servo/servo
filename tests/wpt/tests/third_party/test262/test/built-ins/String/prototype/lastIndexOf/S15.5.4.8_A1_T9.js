// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.lastIndexOf(searchString, position)
es5id: 15.5.4.8_A1_T9
description: >
    Call lastIndexOf(searchString, position) function with
    function(){}() argument of string object
---*/

var __obj = {
  valueOf: function() {},
  toString: void 0
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(undefined) evaluates to "undefined" indexOf(undefined) evaluates to indexOf("undefined")
if (new String(__obj).lastIndexOf(function() {}()) !== 0) {
  throw new Test262Error('#1: __obj = {valueOf:function(){}, toString:void 0}; new String(__obj).lastIndexOf(function(){}()) === 0. Actual: ' + new String(__obj).lastIndexOf(function() {}()));
}
//
//////////////////////////////////////////////////////////////////////////////
