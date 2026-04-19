// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
es6id: B.2.2.1.2
description: Property descriptor for Object.prototype.__proto__
info: |
    Object.prototype.__proto__ is an accessor property with attributes {
    [[Enumerable]]: false, [[Configurable]]: true }. The [[Get]] and [[Set]]
    attributes are defined as follows:
includes: [propertyHelper.js]
features: [__proto__]
---*/

var desc = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__');

assert.sameValue(desc.value, undefined, '`value` property');
assert.sameValue(typeof desc.get, 'function', '`get` property');
assert.sameValue(typeof desc.set, 'function', '`set` property');

verifyProperty(Object.prototype, "__proto__", {
  enumerable: false,
  configurable: true
});
