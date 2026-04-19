// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-constructor
description: >
  The WeakRef constructor is the %WeakRef% intrinsic object and the initial
  value of the WeakRef property of the global object.
features: [WeakRef]
---*/

assert.sameValue(
  typeof WeakRef, 'function',
  'typeof WeakRef is function'
);
