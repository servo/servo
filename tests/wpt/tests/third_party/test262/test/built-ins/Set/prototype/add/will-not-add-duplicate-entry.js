// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: >
    Set.prototype.add ( value )

    ...
    4. Let entries be the List that is the value of S’s [[SetData]] internal slot.
    5. Repeat for each e that is an element of entries,
      a. If e is not empty and SameValueZero(e, value) is true, then
        i. Return S.
    6. If value is −0, let value be +0.
    7. Append value as the last element of entries.
    ...

---*/

var s = new Set();

assert.sameValue(s.size, 0, "The value of `s.size` is `0`");

s.add(1);
s.add(1);

assert.sameValue(s.size, 1, "The value of `s.size` is `1`, after executing `s.add(1); s.add(1);`");
