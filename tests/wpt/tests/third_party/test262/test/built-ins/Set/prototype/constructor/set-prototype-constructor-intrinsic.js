// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
    Set.prototype.constructor

    The initial value of Set.prototype.constructor is the intrinsic object %Set%.

---*/

assert.sameValue(
  Set.prototype.constructor,
  Set,
  "The value of `Set.prototype.constructor` is `Set`"
);
