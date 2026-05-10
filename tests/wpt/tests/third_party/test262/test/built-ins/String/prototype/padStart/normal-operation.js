// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padstart
description: String#padStart should work in the general case
author: Jordan Harband
---*/

assert.sameValue('abc'.padStart(7, 'def'), 'defdabc');
assert.sameValue('abc'.padStart(5, '*'), '**abc');

// surrogate pairs
assert.sameValue('abc'.padStart(6, '\uD83D\uDCA9'), '\uD83D\uDCA9\uD83Dabc');
