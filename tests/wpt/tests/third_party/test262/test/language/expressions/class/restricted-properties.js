// Copyright (C) 2015 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions created using ClassExpression syntactic form do not
    have own properties "caller" or "arguments", but inherit them from
    %FunctionPrototype%.
es6id: 16.1
---*/

var BaseClass = class {};

assert.sameValue(
  BaseClass.hasOwnProperty('caller'), false, 'No "caller" own property'
);
assert.sameValue(
  BaseClass.hasOwnProperty('arguments'), false, 'No "arguments" own property'
);

assert.throws(TypeError, function() {
  return BaseClass.caller;
});

assert.throws(TypeError, function() {
  BaseClass.caller = {};
});

assert.throws(TypeError, function() {
  return BaseClass.arguments;
});

assert.throws(TypeError, function() {
  BaseClass.arguments = {};
});

var SubClass = class extends BaseClass {};

assert.sameValue(
  SubClass.hasOwnProperty('caller'), false, 'No "caller" own property'
);
assert.sameValue(
  SubClass.hasOwnProperty('arguments'), false, 'No "arguments" own property'
);

assert.throws(TypeError, function() {
  return SubClass.caller;
});

assert.throws(TypeError, function() {
  SubClass.caller = {};
});

assert.throws(TypeError, function() {
  return SubClass.arguments;
});

assert.throws(TypeError, function() {
  SubClass.arguments = {};
});
