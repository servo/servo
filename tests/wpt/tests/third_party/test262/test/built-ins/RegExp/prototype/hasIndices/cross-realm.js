// Copyright (C) 2021 Ron Buckton and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.hasindices
description: Invoked on a cross-realm object
info: |
    get RegExp.prototype.hasIndices

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, then
      a. If SameValue(R, %RegExpPrototype%) is true, return undefined.
      b. Otherwise, throw a TypeError exception.
features: [regexp-match-indices, cross-realm]
---*/

var hasIndices = Object.getOwnPropertyDescriptor(RegExp.prototype, 'hasIndices').get;
var other = $262.createRealm().global;
var otherRegExpProto = other.RegExp.prototype;
var otherRegExpGetter = Object.getOwnPropertyDescriptor(otherRegExpProto, 'hasIndices').get;

assert.throws(TypeError, function() {
  hasIndices.call(otherRegExpProto);
}, 'cross-realm RegExp.prototype');

assert.throws(other.TypeError, function() {
  otherRegExpGetter.call(RegExp.prototype);
}, 'cross-realm RegExp.prototype getter method against primary realm RegExp.prototype');
