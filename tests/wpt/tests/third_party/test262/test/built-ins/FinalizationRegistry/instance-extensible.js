// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-target
description: Instances of FinalizationRegistry are extensible
info: |
  FinalizationRegistry ( cleanupCallback )

  ...
  3. Let finalizationRegistry be ? OrdinaryCreateFromConstructor(NewTarget,  "%FinalizationRegistryPrototype%", « [[Realm]], [[CleanupCallback]], [[Cells]], [[IsFinalizationRegistryCleanupJobActive]] »).
  ...
  9. Return finalizationRegistry.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  ObjectCreate ( proto [ , internalSlotsList ] )

  4. Set obj.[[Prototype]] to proto.
  5. Set obj.[[Extensible]] to true.
  6. Return obj.
features: [FinalizationRegistry]
---*/

var finalizationRegistry = new FinalizationRegistry(function() {});
assert.sameValue(Object.isExtensible(finalizationRegistry), true);
