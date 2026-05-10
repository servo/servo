// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T10
description: >
    Call replace (searchValue, replaceValue) function with object and
    function arguments of string. Object have overrided toString
    function
---*/

var __obj = {
  toString: function() {
    return "\u0041B";
  }
};

var __str = "ABB\u0041BABAB";

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__str.replace(__obj, function() {
    return x;
  }) !== "undefinedBABABAB") {
  throw new Test262Error('#1: var x; var __obj = {toString:function(){return "\u0041B";}}; var __str = "ABB\u0041BABAB"; replace(__obj, function(){return x;}) === "undefinedBABABAB". Actual: ' + __str.replace(__obj, function() {
    return x;
  }));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
