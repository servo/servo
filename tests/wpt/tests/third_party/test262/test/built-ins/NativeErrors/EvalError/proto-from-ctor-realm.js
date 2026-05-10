// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-nativeerror
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  NativeError ( message )

  1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
  2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%NativeErrorPrototype%", « [[ErrorData]] »).
  ...
  4. Return O.

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
features: [cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var err;

newTarget.prototype = undefined;
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is null');

newTarget.prototype = false;
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = 'str';
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 0;
err = Reflect.construct(EvalError, [], newTarget);
assert.sameValue(Object.getPrototypeOf(err), other.EvalError.prototype, 'newTarget.prototype is a Number');
