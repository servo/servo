// Copyright (C) 2022 Mathias Bynens, Ron Buckton, and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.unicodesets
description: Invoked on an object without an [[OriginalFlags]] internal slot
info: |
    get RegExp.prototype.unicodeSets -> RegExpHasFlag

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, then
      a. If SameValue(R, %RegExpPrototype%) is true, return undefined.
      b. Otherwise, throw a TypeError exception.
features: [regexp-v-flag]
---*/

var unicodeSets = Object.getOwnPropertyDescriptor(RegExp.prototype, 'unicodeSets').get;

assert.throws(TypeError, function() {
  unicodeSets.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  unicodeSets.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  unicodeSets.call(arguments);
}, 'arguments object');

assert.throws(TypeError, function() {
  unicodeSets.call(() => {});
}, 'function object');
