// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Use DataView.prototype if newTarget's prototype is not an Object
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  12. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DataViewPrototype%",
  « [[DataView]], [[ViewedArrayBuffer]], [[ByteLength]], [[ByteOffset]] »).
  ...
  17. Return O.

  9.1.13 OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ ,
  internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor,
  intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  9.1.15 GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Let proto be realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
  ...
features: [Reflect.construct]
---*/

var buffer = new ArrayBuffer(8);

function newTarget() {}
newTarget.prototype = null;

var sample = Reflect.construct(DataView, [buffer, 0], newTarget);

assert.sameValue(sample.constructor, DataView);
assert.sameValue(Object.getPrototypeOf(sample), DataView.prototype);
