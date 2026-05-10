// Copyright 2016 Leonardo Balter. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Throws a TypeError if this is not Object
features: [Symbol]
---*/

var formatToParts = Intl.DateTimeFormat.prototype.formatToParts;

assert.throws(TypeError, function() {
  formatToParts.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  formatToParts.call(null);
}, "null");

assert.throws(TypeError, function() {
  formatToParts.call(42);
}, "number");

assert.throws(TypeError, function() {
  formatToParts.call("foo");
}, "string");

assert.throws(TypeError, function() {
  formatToParts.call(false);
}, "false");

assert.throws(TypeError, function() {
  formatToParts.call(true);
}, "true");

var s = Symbol('1');
assert.throws(TypeError, function() {
  formatToParts.call(s);
}, "symbol");
