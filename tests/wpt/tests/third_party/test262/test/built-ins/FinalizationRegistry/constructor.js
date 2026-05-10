// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-constructor
description: >
  The FinalizationRegistry constructor is the %FinalizationRegistry% intrinsic object and the initial
  value of the FinalizationRegistry property of the global object.
features: [FinalizationRegistry]
---*/

assert.sameValue(
  typeof FinalizationRegistry, 'function',
  'typeof FinalizationRegistry is function'
);
