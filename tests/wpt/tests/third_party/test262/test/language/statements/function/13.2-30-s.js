// Copyright (C) 2015 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
 description: >
    Functions created using Function.prototype.bind() do not have own
    properties "caller" or "arguments", but inherit them from
    %FunctionPrototype%.
---*/

function target() {}
var self = {};
var bound = target.bind(self);

assert.sameValue(bound.hasOwnProperty('caller'), false, 'Functions created using Function.prototype.bind() do not have own property "caller"');
assert.sameValue(bound.hasOwnProperty('arguments'), false, 'Functions created using Function.prototype.bind() do not have own property "arguments"');

assert.throws(TypeError, function() {
  return bound.caller;
});

assert.throws(TypeError, function() {
  bound.caller = {};
});

assert.throws(TypeError, function() {
  return bound.arguments;
});

assert.throws(TypeError, function() {
  bound.arguments = {};
});
