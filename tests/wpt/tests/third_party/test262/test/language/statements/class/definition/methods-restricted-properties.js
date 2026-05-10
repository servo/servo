// Copyright (C) 2015 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions created using MethodDefinition syntactic form do not have own
    properties "caller" or "arguments", but inherit them from
    %FunctionPrototype%.
es6id: 16.1
---*/

class Class {
  method() {}
  get accessor() {}
  set accessor(x) {}
};

var instance = new Class;
var accessor = Object.getOwnPropertyDescriptor(Class.prototype, "accessor");

assert.sameValue(
  instance.method.hasOwnProperty('caller'),
  false,
  'No "caller" own property (method)'
);
assert.sameValue(
  instance.method.hasOwnProperty('arguments'),
  false,
  'No "arguments" own property (method)'
);
assert.sameValue(
  accessor.get.hasOwnProperty('caller'),
  false,
  'No "caller" own property ("get" accessor)'
);
assert.sameValue(
  accessor.get.hasOwnProperty('arguments'),
  false,
  'No "arguments" own property ("get" accessor)'
);
assert.sameValue(
  accessor.set.hasOwnProperty('caller'),
  false,
  'No "caller" own property ("set" accessor)'
);
assert.sameValue(
  accessor.set.hasOwnProperty('arguments'),
  false,
  'No "arguments" own property ("set" accessor)'
);

// --- Test method restricted properties throw

assert.throws(TypeError, function() {
  return instance.method.caller;
});

assert.throws(TypeError, function() {
  instance.method.caller = {};
});

assert.throws(TypeError, function() {
  return instance.method.arguments;
});

assert.throws(TypeError, function() {
  instance.method.arguments = {};
});

// --- Test getter restricted properties throw

assert.throws(TypeError, function() {
  return accessor.get.caller;
});

assert.throws(TypeError, function() {
  accessor.get.caller = {};
});

assert.throws(TypeError, function() {
  return accessor.get.arguments;
});

assert.throws(TypeError, function() {
  accessor.get.arguments = {};
});

// --- Test setter restricted properties throw

assert.throws(TypeError, function() {
  return accessor.set.caller;
});

assert.throws(TypeError, function() {
  accessor.set.caller = {};
});

assert.throws(TypeError, function() {
  return accessor.set.arguments;
});

assert.throws(TypeError, function() {
  accessor.set.arguments = {};
});
