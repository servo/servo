// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: >
  Return false if entry wasn't in the WeakSet.
info: |
  WeakSet.prototype.delete ( value )

  ...
  7. Return false.

---*/

var s = new WeakSet();

assert.sameValue(s.delete({}), false);
