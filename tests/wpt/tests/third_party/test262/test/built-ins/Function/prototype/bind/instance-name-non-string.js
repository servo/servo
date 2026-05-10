// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.2.3.2
description: >
    Assignment of function `name` attribute (target has non-string name)
info: |
    12. Let targetName be Get(Target, "name").
    13. ReturnIfAbrupt(targetName).
    14. If Type(targetName) is not String, let targetName be the empty string.
    15. Perform SetFunctionName(F, targetName, "bound").
includes: [propertyHelper.js]
features: [Symbol]
---*/

var target;

target = Object.defineProperty(function() {}, 'name', {
  value: undefined
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');

target = Object.defineProperty(function() {}, 'name', {
  value: null
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');

target = Object.defineProperty(function() {}, 'name', {
  value: true
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');

target = Object.defineProperty(function() {}, 'name', {
  value: Symbol('s')
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');

target = Object.defineProperty(function() {}, 'name', {
  value: 23
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');

target = Object.defineProperty(function() {}, 'name', {
  value: {}
});

assert.sameValue(target.bind().name, 'bound ');
verifyNotEnumerable(target.bind(), 'name');
verifyNotWritable(target.bind(), 'name');
verifyConfigurable(target.bind(), 'name');
