// Copyright (C) 2015 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    ECMAScript Function objects defined using syntactic constructors
    in strict mode code do not have own properties "caller" or
    "arguments" other than those that are created by applying the
    AddRestrictedFunctionProperties abstract operation to the function.
flags: [onlyStrict]
es6id: 16.1
---*/

function func() {}

assert.throws(TypeError, function() {
  return func.caller;
}, 'return func.caller throws a TypeError exception');

assert.throws(TypeError, function() {
  func.caller = {};
}, 'func.caller = {} throws a TypeError exception');

assert.throws(TypeError, function() {
  return func.arguments;
}, 'return func.arguments throws a TypeError exception');

assert.throws(TypeError, function() {
  func.arguments = {};
}, 'func.arguments = {} throws a TypeError exception');

var newfunc = new Function('"use strict"');

assert.sameValue(newfunc.hasOwnProperty('caller'), false, 'newfunc.hasOwnProperty(\'caller\') must return false');
assert.sameValue(newfunc.hasOwnProperty('arguments'), false, 'newfunc.hasOwnProperty(\'arguments\') must return false');

assert.throws(TypeError, function() {
  return newfunc.caller;
}, 'return newfunc.caller throws a TypeError exception');

assert.throws(TypeError, function() {
  newfunc.caller = {};
}, 'newfunc.caller = {} throws a TypeError exception');

assert.throws(TypeError, function() {
  return newfunc.arguments;
}, 'return newfunc.arguments throws a TypeError exception');

assert.throws(TypeError, function() {
  newfunc.arguments = {};
}, 'newfunc.arguments = {} throws a TypeError exception');
