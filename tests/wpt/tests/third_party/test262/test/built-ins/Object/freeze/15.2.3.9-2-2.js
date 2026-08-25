// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-2
description: Object.freeze - inherited data properties are not frozen
---*/

var proto = {};

Object.defineProperty(proto, "Father", {
  value: 10,
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
Object.freeze(child);

var beforeDeleted = proto.hasOwnProperty("Father");
delete proto.Father;
var afterDeleted = proto.hasOwnProperty("Father");

assert(beforeDeleted, 'beforeDeleted !== true');
assert.sameValue(afterDeleted, false, 'afterDeleted');
