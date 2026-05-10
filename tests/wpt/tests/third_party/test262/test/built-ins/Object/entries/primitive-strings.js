// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries accepts string primitives.
author: Jordan Harband
---*/

var result = Object.entries('abc');

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 3, 'result has 3 items');

assert.sameValue(result[0][0], '0', 'first entry has key "0"');
assert.sameValue(result[0][1], 'a', 'first entry has value "a"');
assert.sameValue(result[1][0], '1', 'second entry has key "1"');
assert.sameValue(result[1][1], 'b', 'second entry has value "b"');
assert.sameValue(result[2][0], '2', 'third entry has key "2"');
assert.sameValue(result[2][1], 'c', 'third entry has value "c"');
