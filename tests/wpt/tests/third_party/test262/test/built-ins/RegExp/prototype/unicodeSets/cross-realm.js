// Copyright (C) 2022 Mathias Bynens, Ron Buckton, and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.unicodesets
description: RegExp#unicodeSets invoked on a cross-realm object
info: |
    get RegExp.prototype.unicodeSets -> RegExpHasFlag

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, then
      a. If SameValue(R, %RegExpPrototype%) is true, return undefined.
      b. Otherwise, throw a TypeError exception.
features: [regexp-v-flag, cross-realm]
---*/

var unicodeSets = Object.getOwnPropertyDescriptor(RegExp.prototype, 'unicodeSets').get;
var other = $262.createRealm().global;
var otherRegExpProto = other.RegExp.prototype;
var otherRegExpGetter = Object.getOwnPropertyDescriptor(otherRegExpProto, 'unicodeSets').get;

assert.throws(TypeError, function() {
  unicodeSets.call(otherRegExpProto);
}, 'cross-realm RegExp.prototype');

assert.throws(other.TypeError, function() {
  otherRegExpGetter.call(RegExp.prototype);
}, 'cross-realm RegExp.prototype getter method against primary realm RegExp.prototype');
