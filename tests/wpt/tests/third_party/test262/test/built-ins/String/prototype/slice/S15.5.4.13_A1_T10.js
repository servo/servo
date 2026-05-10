// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T10
description: >
    Arguments are object and function call, and instance is String,
    object have overrided valueOf function
---*/

var __obj = {
  valueOf: function() {
    return 2;
  }
};

var __str = "\u0035ABBBABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.slice(__obj, function() {
    return __str.slice(0, 1);
  }()) !== "BBB") {
  throw new Test262Error('#1: var x; var __obj = {valueOf:function(){return 2;}}; var __str = "\u0035ABBBABAB"; __str.slice(__obj, function(){return __str.slice(0,1);}()) === "BBB". Actual: ' + __str.slice(__obj, function() {
    return __str.slice(0, 1);
  }()));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
