// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invoked on a cross-realm object without an [[OriginalFlags]] internal slot
es6id: 21.2.5.12
info: |
    21.2.5.12 get RegExp.prototype.sticky

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, throw a TypeError
       exception.
features: [cross-realm]
---*/

var sticky = Object.getOwnPropertyDescriptor(RegExp.prototype, 'sticky').get;
var other = $262.createRealm().global;
var otherRegExpProto = other.RegExp.prototype;
var otherRegExpGetter = Object.getOwnPropertyDescriptor(otherRegExpProto, 'sticky').get;

assert.throws(TypeError, function() {
  sticky.call(otherRegExpProto);
}, 'cross-realm RegExp.prototype');

assert.throws(other.TypeError, function() {
  otherRegExpGetter.call(RegExp.prototype);
}, 'cross-realm RegExp.prototype getter method against primary realm RegExp.prototype');
