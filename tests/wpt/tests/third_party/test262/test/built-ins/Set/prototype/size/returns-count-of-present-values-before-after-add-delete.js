// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-set.prototype.size
description: >
    get Set.prototype.size

    5. Let count be 0.
    6. For each e that is an element of entries
      a. If e is not empty, set count to count+1.

    Before and after add(), delete()
---*/

var s = new Set();

assert.sameValue(s.size, 0, "The value of `s.size` is `0`");

s.add(0);

assert.sameValue(s.size, 1, "The value of `s.size` is `1`, after executing `s.add(0)`");

s.delete(0);

assert.sameValue(s.size, 0, "The value of `s.size` is `0`, after executing `s.delete(0)`");
