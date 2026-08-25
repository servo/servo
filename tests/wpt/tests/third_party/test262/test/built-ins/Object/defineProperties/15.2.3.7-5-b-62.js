// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-62
description: >
    Object.defineProperties - 'configurable' property of 'descObj' is
    inherited data property (8.10.5 step 4.a)
---*/

var obj = {};
var proto = {
  configurable: true
};
var Con = function() {};
Con.prototype = proto;
var descObj = new Con();

Object.defineProperties(obj, {
  prop: descObj
});

var result1 = obj.hasOwnProperty("prop");
delete obj.prop;
var result2 = obj.hasOwnProperty("prop");

assert.sameValue(result1, true, 'result1');
assert.sameValue(result2, false, 'result2');
