// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-constructor
description: >
  The WeakSet constructor is the %WeakSet% intrinsic object and the initial
  value of the WeakSet property of the global object.
---*/

assert.sameValue(
  typeof WeakSet, 'function',
  'typeof WeakSet is "function"'
);
