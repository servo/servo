// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - 'P' is own data property that overrides an inherited
    accessor property
includes: [propertyHelper.js]
---*/

var proto = {};

Object.defineProperty(proto, "foo", {
  get: function() {
    return 0;
  },
  configurable: true
});

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var obj = new ConstructFun();
Object.defineProperty(obj, "foo", {
  value: 10,
  configurable: true
});

assert(Object.isExtensible(obj));
Object.seal(obj);

verifyProperty(obj, "foo", {
  value: 10,
  configurable: false,
});
