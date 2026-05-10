// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
    Set.prototype.add ( value )

    1. Let S be this value.
    ...
    5. Repeat for each e that is an element of entries,
      a. If e is not empty and SameValueZero(e, value) is true, then
        i. Return S.
    ...

---*/

var s = new Set([1]);

assert.sameValue(s.add(1), s, "`s.add(1)` returns `s`");
