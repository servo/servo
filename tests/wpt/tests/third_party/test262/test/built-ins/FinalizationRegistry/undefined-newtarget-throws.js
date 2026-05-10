// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-target
description: >
  Throws a TypeError if NewTarget is undefined.
info: |
  FinalizationRegistry ( cleanupCallback )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. If IsCallable(cleanupCallback) is false, throw a TypeError exception.
  ...
features: [FinalizationRegistry]
---*/

assert.sameValue(
  typeof FinalizationRegistry, 'function',
  'typeof FinalizationRegistry is function'
);

assert.throws(TypeError, function() {
  FinalizationRegistry();
});

assert.throws(TypeError, function() {
  FinalizationRegistry(function() {});
});
