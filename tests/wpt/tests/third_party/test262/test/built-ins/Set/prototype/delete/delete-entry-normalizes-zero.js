// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.delete
description: >
    Set.prototype.delete ( value )

    ...
    4. Let entries be the List that is the value of S’s [[SetData]] internal slot.
    5. Repeat for each e that is an element of entries,
      a. If e is not empty and SameValueZero(e, value) is true, then
      b. Replace the element of entries whose value is e with an element whose value is empty.
      c. Return true.
    ...

---*/

var s = new Set([-0]);

assert.sameValue(s.size, 1, "The value of `s.size` is `1`");

var result = s.delete(+0);

assert.sameValue(s.size, 0, "The value of `s.size` is `0`, after executing `s.delete(-0)`");
assert.sameValue(result, true, "The result of `s.delete(+0)` is `true`");
