// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when "this" value is not Object-coercible
info: |
    1. Let O be ? ToObject(this value).
features: [__setter__]
---*/

var __defineSetter__ = Object.prototype.__defineSetter__;
var noop = function() {};
var toStringCount = 0;
var key = {
  toString: function() {
    toStringCount += 1;
  }
};

assert.sameValue(typeof __defineSetter__, 'function');

assert.throws(TypeError, function() {
  __defineSetter__.call(undefined, key, noop);
}, 'undefined');

assert.throws(TypeError, function() {
  __defineSetter__.call(null, key, noop);
}, 'null');

assert.sameValue(toStringCount, 0);
