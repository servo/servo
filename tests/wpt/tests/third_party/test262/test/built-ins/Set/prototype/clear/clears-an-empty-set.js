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

var s = new Set();

var result = s.clear();

assert.sameValue(s.size, 0, "The value of `s.size` is `0`");
assert.sameValue(result, undefined, "The result of `s.clear()` is `undefined`");
