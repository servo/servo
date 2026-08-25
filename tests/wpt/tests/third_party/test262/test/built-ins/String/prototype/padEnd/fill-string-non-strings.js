// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padend
description: String#padEnd should stringify a non-string fillString value
author: Jordan Harband
---*/

assert.sameValue('abc'.padEnd(10, false), 'abcfalsefa');
assert.sameValue('abc'.padEnd(10, true), 'abctruetru');
assert.sameValue('abc'.padEnd(10, null), 'abcnullnul');
assert.sameValue('abc'.padEnd(10, 0), 'abc0000000');
assert.sameValue('abc'.padEnd(10, -0), 'abc0000000');
assert.sameValue('abc'.padEnd(10, NaN), 'abcNaNNaNN');
