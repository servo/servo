// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp)
es5id: 15.5.4.12_A1_T9
description: >
    Argument is function call, and instance is String object with
    overrided toString and valueOf functions
---*/

var __obj = {
  valueOf: function() {},
  toString: void 0
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(undefined) evaluates to "undefined" search(undefined) evaluates to search("undefined")
if (new String(__obj).search(function() {}()) !== 0) {
  throw new Test262Error('#1: __obj = {valueOf:function(){}, toString:void 0}; new String(__obj).search(function(){}()) === 0. Actual: ' + new String(__obj).search(function() {}()));
}
//
//////////////////////////////////////////////////////////////////////////////
