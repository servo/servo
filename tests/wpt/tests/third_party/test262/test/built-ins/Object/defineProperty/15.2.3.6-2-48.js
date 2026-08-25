// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-48
description: >
    Object.defineProperty - an inherited toString method  is invoked
    when 'P' is an object with an own valueOf and an inherited
    toString methods
---*/

var obj = {};
var toStringAccessed = false;
var valueOfAccessed = false;

var proto = {
  toString: function() {
    toStringAccessed = true;
    return "test";
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
child.valueOf = function() {
  valueOfAccessed = true;
  return "10";
};

Object.defineProperty(obj, child, {});

assert(obj.hasOwnProperty("test"), 'obj.hasOwnProperty("test") !== true');
assert.sameValue(valueOfAccessed, false, 'valueOfAccessed');
assert(toStringAccessed, 'toStringAccessed !== true');
