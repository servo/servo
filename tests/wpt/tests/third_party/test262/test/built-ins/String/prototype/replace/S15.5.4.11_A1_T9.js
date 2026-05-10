// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T9
description: >
    Call replace (searchValue, replaceValue) function with functions
    arguments of new String object
---*/

var __obj = {
  valueOf: function() {},
  toString: void 0
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (new String(__obj).replace(function() {}(), __func) !== "undefined0undefined") {
  throw new Test262Error('#1: __obj = {valueOf:function(){}, toString:void 0}; function __func(a1,a2,a3){return a1+a2+a3;}; new String(__obj).replace(function(){}(),__func) === "undefined0undefined". Actual: ' + new String(__obj).replace(function() {}(), __func));
}
//
//////////////////////////////////////////////////////////////////////////////

function __func(a1, a2, a3) {
  return a1 + a2 + a3;
};
