// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.clear
description: >
    Set.prototype.clear ( )

    ...
    4. Let entries be the List that is the value of S’s [[SetData]] internal slot.
    5. Repeat for each e that is an element of entries,
      a. Replace the element of entries whose value is e with an element whose value is empty.
    ...

---*/

var s = new Set([1, 2, 3]);

assert.sameValue(s.size, 3, "The value of `s.size` is `3`");

var result = s.clear();

assert.sameValue(s.size, 0, "The value of `s.size` is `0`, after executing `s.clear()`");
assert.sameValue(s.has(1), false, "`s.has(1)` returns `false`");
assert.sameValue(s.has(2), false, "`s.has(2)` returns `false`");
assert.sameValue(s.has(3), false, "`s.has(3)` returns `false`");
assert.sameValue(result, undefined, "The result of `s.clear()` is `undefined`");
