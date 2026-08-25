// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T9
description: >
    Arguments are undefined and object, and instance is
    String(object), object have overrided valueOf and toString
    functions
---*/

var __obj = {
  valueOf: function() {},
  toString: void 0
};

//since ToInteger(undefined) yelds 0
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (new String(__obj).slice( /*(function(){})()*/ undefined, __obj) !== "") {
  throw new Test262Error('#1: __obj = {valueOf:function(){}, toString:void 0}; new String(__obj).slice(//*(function(){})()*//undefined,__obj) === "". Actual: ' + new String(__obj).slice( /*(function(){})()*/ undefined, __obj));
}
//
//////////////////////////////////////////////////////////////////////////////
