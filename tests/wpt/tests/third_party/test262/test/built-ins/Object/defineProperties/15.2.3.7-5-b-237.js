// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-237
description: >
    Object.defineProperties - 'set' property of 'descObj' is own
    accessor property without a get function that overrides an
    inherited accessor property (8.10.5 step 8.a)
---*/

var fun = function() {
  return 10;
};
var proto = {};
Object.defineProperty(proto, "set", {
  get: function() {
    return function() {
      return arguments;
    };
  }
});

var Con = function() {};
Con.prototype = proto;

var descObj = new Con();
Object.defineProperty(descObj, "set", {
  set: function() {}
});

descObj.get = fun;

var obj = {};

Object.defineProperties(obj, {
  prop: descObj
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(typeof(desc.set), "undefined", 'typeof (desc.set)');
assert.sameValue(obj.prop, 10, 'obj.prop');
