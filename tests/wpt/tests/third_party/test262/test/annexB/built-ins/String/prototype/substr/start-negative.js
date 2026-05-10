// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: Behavior when "start" is a negative number
info: |
    [...]
    6. If intStart < 0, let intStart be max(size + intStart, 0).
---*/

assert.sameValue('abc'.substr(-1), 'c');
assert.sameValue('abc'.substr(-2), 'bc');
assert.sameValue('abc'.substr(-3), 'abc');
assert.sameValue('abc'.substr(-4), 'abc', 'size + intStart < 0');

assert.sameValue('abc'.substr(-1.1), 'c', 'floating point rounding semantics');
