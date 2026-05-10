// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleLowerCase()
es5id: 15.5.4.17_A1_T13
description: >
    Override toString and valueOf functions, then call
    toLocaleLowerCase() function for this object
---*/

var __obj = {
  toString: function() {
    return {};
  },
  valueOf: function() {
    return 1;
  }
}
__obj.toLocaleLowerCase = String.prototype.toLocaleLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.toLocaleLowerCase() !== "1") {
  throw new Test262Error('#1: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toLocaleLowerCase = String.prototype.toLocaleLowerCase; __obj.toLocaleLowerCase() ==="1". Actual: ' + __obj.toLocaleLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj.toLocaleLowerCase().length !== 1) {
  throw new Test262Error('#2: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; __obj.toLocaleLowerCase = String.prototype.toLocaleLowerCase; __obj.toLocaleLowerCase().length === 1. Actual: ' + __obj.toLocaleLowerCase().length);
}
//
//////////////////////////////////////////////////////////////////////////////
