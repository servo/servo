// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T13
description: >
    Override toString and valueOf functions, then call toLowerCase()
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
__obj.toLowerCase = String.prototype.toLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.toLowerCase() !== "1") {
  throw new Test262Error('#1: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toLowerCase = String.prototype.toLowerCase; __obj.toLowerCase() ==="1". Actual: ' + __obj.toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj.toLowerCase().length !== 1) {
  throw new Test262Error('#2: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toLowerCase = String.prototype.toLowerCase; __obj.toLowerCase().length === 1. Actual: ' + __obj.toLowerCase().length);
}
//
//////////////////////////////////////////////////////////////////////////////
