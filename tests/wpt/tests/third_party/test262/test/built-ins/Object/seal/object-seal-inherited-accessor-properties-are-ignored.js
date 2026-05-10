// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - inherited accessor properties are ignored
---*/

var proto = {};

Object.defineProperty(proto, "Father", {
  get: function() {
    return 10;
  },
  configurable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
var preCheck = Object.isExtensible(child);
Object.seal(child);

var beforeDeleted = proto.hasOwnProperty("Father");
delete proto.Father;
var afterDeleted = proto.hasOwnProperty("Father");

assert(preCheck, 'preCheck !== true');
assert(beforeDeleted, 'beforeDeleted !== true');
assert.sameValue(afterDeleted, false, 'afterDeleted');
