// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: Object.values accepts string primitives.
author: Jordan Harband
---*/

var result = Object.values('abc');

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 3, 'result has 3 items');

assert.sameValue(result[0], 'a', 'first value is "a"');
assert.sameValue(result[1], 'b', 'second value is "b"');
assert.sameValue(result[2], 'c', 'third value is "c"');
