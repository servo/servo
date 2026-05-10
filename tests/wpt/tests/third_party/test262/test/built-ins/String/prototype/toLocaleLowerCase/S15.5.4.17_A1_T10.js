// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleLowerCase()
es5id: 15.5.4.17_A1_T10
description: >
    Call toLocaleLowerCase() function of object with overrode toString
    function
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
}
__obj.toLocaleLowerCase = String.prototype.toLocaleLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.toLocaleLowerCase() !== "ab") {
  throw new Test262Error('#1: var __obj = {toString:function(){return "\u0041B";}}; __obj.toLocaleLowerCase = String.prototype.toLocaleLowerCase; __obj.toLocaleLowerCase() ==="ab". Actual: ' + __obj.toLocaleLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
