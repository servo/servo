// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant
description: >
  OrdinaryCreateFromConstructor returns with an abrupt completion.
info: |
  CreateTemporalInstant ( epochNanoseconds [ , newTarget ] )

  ...
  3. Let object be ? OrdinaryCreateFromConstructor(newTarget,
     "%Temporal.Instant.prototype%", « [[InitializedTemporalInstant]], [[EpochNanoseconds]] »).
  ...

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  ...

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  2. Let proto be ? Get(constructor, "prototype").
  ...

features: [Temporal]
---*/

var newTarget = Object.defineProperty(function(){}.bind(), "prototype", {
  get() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.construct(Temporal.Instant, [0n], newTarget)
});
