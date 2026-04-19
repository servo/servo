// Copyright (C) 2022 Mathias Bynens, Ron Buckton, and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.unicodesets
description: >
    `unicodeSets` accessor invoked on a non-object value
info: |
    get RegExp.prototype.unicodeSets -> RegExpHasFlag

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol, regexp-v-flag]
---*/

var unicodeSets = Object.getOwnPropertyDescriptor(RegExp.prototype, "unicodeSets").get;

assert.throws(TypeError, function() {
  unicodeSets.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  unicodeSets.call(null);
}, "null");

assert.throws(TypeError, function() {
  unicodeSets.call(true);
}, "true");

assert.throws(TypeError, function() {
  unicodeSets.call("string");
}, "string");

assert.throws(TypeError, function() {
  unicodeSets.call(Symbol("s"));
}, "symbol");

assert.throws(TypeError, function() {
  unicodeSets.call(4);
}, "number");

assert.throws(TypeError, function() {
  unicodeSets.call(4n);
}, "bigint");
