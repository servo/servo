// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set.prototype.keys
description: >
    The initial value of the keys property is the same function object as the
    initial value of the values property.
---*/

assert.sameValue(
  Set.prototype.keys,
  Set.prototype.values,
  "The value of `Set.prototype.keys` is `Set.prototype.values`"
);
