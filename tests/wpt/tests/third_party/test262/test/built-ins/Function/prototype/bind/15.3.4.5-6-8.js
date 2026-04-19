// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.5-6-8
description: >
    Function.prototype.bind - F can get own accessor property that
    overrides an inherited accessor property
---*/

var foo = function() {};

var obj = foo.bind({});

Object.defineProperty(Function.prototype, "property", {
  get: function() {
    return 3;
  },
  configurable: true
});

Object.defineProperty(obj, "property", {
  get: function() {
    return 12;
  }
});

assert.sameValue(obj.property, 12, 'obj.property');
