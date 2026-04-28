// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.has
description: >
    Set.prototype.has ( value )

    ...
    6. Return false.

---*/

var s = new Set();

s.add(undefined);
assert.sameValue(s.has(undefined), true, "`s.has(undefined)` returns `true`");

var result = s.delete(undefined);

assert.sameValue(s.has(undefined), false, "`s.has(undefined)` returns `false`");
assert.sameValue(result, true, "The result of `s.delete(undefined)` is `true`");
