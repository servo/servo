// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.constructor
description: >
  The initial value of WeakSet.prototype.constructor is the %WeakSet%
  intrinsic object.
---*/

assert.sameValue(
  WeakSet.prototype.constructor,
  WeakSet,
  'The value of WeakSet.prototype.constructor is "WeakSet"'
);
