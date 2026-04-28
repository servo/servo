// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
    Set.prototype.add ( value )

    1. Let S be this value.
    ...
    8. Return S.

---*/

var s = new Set();

assert.sameValue(s.add(1), s, "`s.add(1)` returns `s`");
