// Copyright (C) 2015 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions created using GeneratorFunction intrinsic function do not have
    own properties "caller" or "arguments", but inherit them from
    %FunctionPrototype%.
features: [generators]
---*/

var Generator = Object.getPrototypeOf(function*() {});
var GeneratorFunction = Generator.constructor;
var generator = new GeneratorFunction();

assert.sameValue(
  generator.hasOwnProperty('caller'), false, 'No "caller" own property'
);
assert.sameValue(
  generator.hasOwnProperty('arguments'), false, 'No "arguments" own property'
);

assert.throws(TypeError, function() {
  return generator.caller;
});

assert.throws(TypeError, function() {
  generator.caller = {};
});

assert.throws(TypeError, function() {
  return generator.arguments;
});

assert.throws(TypeError, function() {
  generator.arguments = {};
});
