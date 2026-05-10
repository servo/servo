// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totemporalinstant
description: >
  Behaviour when "this" value is not an Object
info: |
  Date.prototype.toTemporalInstant ( )

  1. Let dateObject be the this value.
  2. Perform ? RequireInternalSlot(dateObject, [[DateValue]]).
  ...
features: [Temporal, Symbol, BigInt]
---*/

var toTemporalInstant = Date.prototype.toTemporalInstant;
var symbol = Symbol();

assert.sameValue(typeof toTemporalInstant, "function");

assert.throws(TypeError, function() {
  toTemporalInstant.call(0);
}, "number");

assert.throws(TypeError, function() {
  toTemporalInstant.call(true);
}, "boolean");

assert.throws(TypeError, function() {
  toTemporalInstant.call(null);
}, "null");

assert.throws(TypeError, function() {
  toTemporalInstant.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  toTemporalInstant.call("");
}, "string");

assert.throws(TypeError, function() {
  toTemporalInstant.call(symbol);
}, "symbol");

assert.throws(TypeError, function() {
  toTemporalInstant.call(0n);
}, "bigint");
