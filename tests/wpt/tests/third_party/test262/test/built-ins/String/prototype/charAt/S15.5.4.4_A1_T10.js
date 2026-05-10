// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charAt(pos)
es5id: 15.5.4.4_A1_T10
description: Call charAt() function with object argument
---*/

var __obj = {
  toString: function() {
    return 1;
  }
}
var __str = "lego";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.charAt(__obj) !== "e") {
  throw new Test262Error('#1: var __obj = {toString:function(){return 1;}}; var __str = "lego"; __str.charAt(__obj) === "e". Actual: ' + __str.charAt(__obj));
}
//
//////////////////////////////////////////////////////////////////////////////
