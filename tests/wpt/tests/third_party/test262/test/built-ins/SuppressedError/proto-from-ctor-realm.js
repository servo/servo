// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  SuppressedError ( error, suppressed, message )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
  2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%SuppressedError.prototype%", « [[ErrorData]] »).
  ...
  6. Return O.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, 'prototype').
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [explicit-resource-management, cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var err;

newTarget.prototype = undefined;
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = -1;
err = Reflect.construct(SuppressedError, [[]], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.SuppressedError.prototype, 'newTarget.prototype is a Number');
