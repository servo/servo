// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.concat([,[...]])
es5id: 15.5.4.6_A1_T10
description: Call concat([,[...]]) function with object arguments
---*/

var __obj = {
  toString: function() {
    return "\u0041";
  },
  valueOf: function() {
    return "_\u0041_";
  }
}
var __obj2 = {
  toString: function() {
    return true;
  }
}
var __obj3 = {
  toString: function() {
    return 42;
  }
}
var __str = "lego";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.concat(__obj) !== "legoA") {
  throw new Test262Error('#1: var x; var __obj = {toString:function(){return "\u0041";}, valueOf:function(){return "_\u0041_";}}; var __str = "lego"; __str.concat(__obj) === "legoA". Actual: ' + __str.concat(__obj));
}
if (__str.concat(__obj, __obj2, __obj3, x) !== "legoAtrue42undefined") {
  throw new Test262Error('#2: var x; var __obj = {toString:function(){return "\u0041";}, valueOf:function(){return "_\u0041_";}}; var __obj2 = {toString:function(){return true;}}; var __obj3 = {toString:function(){return 42;}}; var __str = "lego"; __str.concat(__obj, __obj2, __obj3, x) === "legoAtrue42undefined". Actual: ' + __str.concat(__obj, __obj2, __obj3, x));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
