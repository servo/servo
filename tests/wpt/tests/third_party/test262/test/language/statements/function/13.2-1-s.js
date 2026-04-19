// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-1-s
description: >
    StrictMode -  Writing or reading from a property named 'caller' of
    function objects is allowed under both strict and normal modes.
---*/

var foo = function () {
    this.caller = 12;
}
var obj = new foo();

assert.sameValue(obj.caller, 12, 'obj.caller');
