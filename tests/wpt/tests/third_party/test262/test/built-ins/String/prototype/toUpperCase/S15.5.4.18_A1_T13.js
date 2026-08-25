// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase()
es5id: 15.5.4.18_A1_T13
description: >
    Override toString and valueOf functions, then call toUpperCase()
    function for this object
---*/

var __obj = {
  toString: function() {
    return {};
  },
  valueOf: function() {
    return 1;
  }
}
__obj.toUpperCase = String.prototype.toUpperCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.toUpperCase() !== "1") {
  throw new Test262Error('#1: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toUpperCase = String.prototype.toUpperCase; __obj.toUpperCase() ==="1". Actual: ' + __obj.toUpperCase());
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj.toUpperCase().length !== 1) {
  throw new Test262Error('#2: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toUpperCase = String.prototype.toUpperCase; __obj.toUpperCase().length === 1. Actual: ' + __obj.toUpperCase().length);
}
//
//////////////////////////////////////////////////////////////////////////////
