// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.sort
description: throws on a non-undefined non-function
info: |
  22.1.3.25 Array.prototype.sort ( comparefn )

  Upon entry, the following steps are performed to initialize evaluation
  of the sort function:

  ...
  1. If _comparefn_ is not *undefined* and IsCallable(_comparefn_) is *false*, throw a *TypeError* exception.
  ...
features: [Symbol]
---*/

var sample = [1];
var poisoned = {
  get length() {
    throw new Test262Error("IsCallable(comparefn) should be observed before this.length");
  }
};

assert.throws(TypeError, function() {
  sample.sort(null);
}, "sample.sort(null);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, null);
}, "[].sort.call(poisoned, null);");

assert.throws(TypeError, function() {
  sample.sort(true);
}, "sample.sort(true);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, true);
}, "[].sort.call(poisoned, true);");

assert.throws(TypeError, function() {
  sample.sort(false);
}, "sample.sort(false);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, false);
}, "[].sort.call(poisoned, false);");

assert.throws(TypeError, function() {
  sample.sort('');
}, "sample.sort('');");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, '');
}, "[].sort.call(poisoned, '');");

assert.throws(TypeError, function() {
  sample.sort(/a/g);
}, "sample.sort(/a/g);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, /a/g);
}, "[].sort.call(poisoned, /a/g);");

assert.throws(TypeError, function() {
  sample.sort(42);
}, "sample.sort(42);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, 42);
}, "[].sort.call(poisoned, 42);");

assert.throws(TypeError, function() {
  sample.sort([]);
}, "sample.sort([]);");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, []);
}, "[].sort.call(poisoned, []);");

assert.throws(TypeError, function() {
  sample.sort({});
}, "sample.sort({});");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, {});
}, "[].sort.call(poisoned, {});");

assert.throws(TypeError, function() {
  sample.sort(Symbol());
}, "sample.sort(Symbol());");

assert.throws(TypeError, function() {
  [].sort.call(poisoned, Symbol());
}, "[].sort.call(poisoned, Symbol());");
