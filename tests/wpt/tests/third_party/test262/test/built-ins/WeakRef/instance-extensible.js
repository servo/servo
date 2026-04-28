// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: Instances of WeakRef are extensible
info: |
  WeakRef( target )

  ...
  3. Let weakRef be ? OrdinaryCreateFromConstructor(NewTarget,  "%WeakRefPrototype%", « [[Target]] »).
  4. Perfom ! KeepDuringJob(target).
  5. Set weakRef.[[Target]] to target.
  6. Return weakRef.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  ObjectCreate ( proto [ , internalSlotsList ] )

  4. Set obj.[[Prototype]] to proto.
  5. Set obj.[[Extensible]] to true.
  6. Return obj.
features: [WeakRef, Reflect]
---*/

var wr = new WeakRef({});
assert.sameValue(Object.isExtensible(wr), true);
