// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    If a property has a corresponding target object property then applying the
    Property Descriptor of the property to the target object using
    [[DefineOwnProperty]] will not throw an exception.
features: [Proxy, Reflect]
includes: [propertyHelper.js]
---*/

var target = {};
var p = new Proxy(target, {
  defineProperty: function(t, prop, desc) {
    return Object.defineProperty(t, prop, desc);
  }
});

var result = Reflect.defineProperty(p, "attr", {
  configurable: true,
  enumerable: true,
  writable: true,
  value: 1
});

assert.sameValue(result, true, "result === true");

verifyEqualTo(target, "attr", 1);
verifyProperty(target, "attr", {
  writable: true,
  enumerable: true,
  configurable: true,
});

result = Reflect.defineProperty(p, "attr", {
  configurable: false,
  enumerable: false,
  writable: false,
  value: 2
});

assert.sameValue(result, true, "result === true");

verifyEqualTo(target, "attr", 2);
verifyProperty(target, "attr", {
  writable: false,
  enumerable: false,
  configurable: false,
});
