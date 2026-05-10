// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array-len
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  Array ( len )

  ...
  3. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
  4. Let proto be ? GetPrototypeFromConstructor(newTarget, "%Array.prototype%").
  5. Let array be ! ArrayCreate(0, proto).
  ...
  9. Return array.

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [cross-realm, Reflect, Symbol]
---*/

var other = $262.createRealm().global;
var newTarget = new other.Function();
var arr;

newTarget.prototype = undefined;
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');

newTarget.prototype = null;
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');

newTarget.prototype = true;
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');

newTarget.prototype = '';
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');

newTarget.prototype = Symbol();
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');

newTarget.prototype = 0;
arr = Reflect.construct(Array, [1], newTarget);
assert.sameValue(Object.getPrototypeOf(arr), other.Array.prototype, 'Object.getPrototypeOf(Reflect.construct(Array, [1], newTarget)) returns other.Array.prototype');
