// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totemporalinstant
description: >
  Behaviour when "this" value is an Object without a [[DateValue]] internal slot
info: |
  Date.prototype.toTemporalInstant ( )

  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  ...
features: [Temporal]
---*/

var toTemporalInstant = Date.prototype.toTemporalInstant;

var args = (function() {
  return arguments;
}());

assert.sameValue(typeof toTemporalInstant, "function");

assert.throws(TypeError, function() {
  toTemporalInstant.call({});
}, "ordinary object");

assert.throws(TypeError, function() {
  toTemporalInstant.call([]);
}, "array exotic object");

assert.throws(TypeError, function() {
  toTemporalInstant.call(args);
}, "arguments exotic object");

assert.throws(TypeError, function() {
  toTemporalInstant.call(function(){});
}, "function object");

assert.throws(TypeError, function() {
  toTemporalInstant.call(Date.prototype);
}, "Date.prototype");
