// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.7
description: >
    A property cannot be reported as non-existent, if it exists as an own
    property of the target object and the target object is not extensible.
info: |
    [[HasProperty]] (P)

    ...
    11. If booleanTrapResult is false, then
        a. Let targetDesc be target.[[GetOwnProperty]](P).
        b. ReturnIfAbrupt(targetDesc).
        c. If targetDesc is not undefined, then
            ...
            ii. Let extensibleTarget be IsExtensible(target).
            ...
            iv. If extensibleTarget is false, throw a TypeError exception.
    ...
flags: [noStrict]
features: [Proxy]
---*/

var target = {};
var handler = {
  has: function(t, prop) {
    return 0;
  }
};
var p = new Proxy(target, handler);

Object.defineProperty(target, 'attr', {
  configurable: true,
  value: 1
});

Object.preventExtensions(target);

assert.throws(TypeError, function() {
  with(p) {
    (attr);
  }
});
