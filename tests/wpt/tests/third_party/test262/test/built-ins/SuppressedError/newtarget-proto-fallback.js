// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  Fallback to the NewTarget's [[Prototype]] if the prototype property is not an object
info: |
  SuppressedError ( error, suppressed, message )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
  2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%SuppressedError.prototype%", « [[ErrorData]], [[AggregateErrors]] »).
  ...
  6. Return O.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  Return proto.
features: [explicit-resource-management, Symbol]
---*/

const values = [
  undefined,
  null,
  42,
  false,
  true,
  Symbol(),
  'string',
  SuppressedError.prototype,
];

const NewTarget = new Function();

for (const value of values) {
  const NewTargetProxy = new Proxy(NewTarget, {
    get(t, p) {
      if (p === 'prototype') {
        return value;
      }
      return t[p];
    }
  });

  const error = Reflect.construct(SuppressedError, [], NewTargetProxy);
  assert.sameValue(Object.getPrototypeOf(error), SuppressedError.prototype);
}
