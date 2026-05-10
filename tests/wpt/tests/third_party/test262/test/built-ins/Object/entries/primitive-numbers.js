// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries accepts number primitives.
author: Jordan Harband
---*/

assert.sameValue(Object.entries(0).length, 0, '0 has zero entries');
assert.sameValue(Object.entries(-0).length, 0, '-0 has zero entries');
assert.sameValue(Object.entries(Infinity).length, 0, 'Infinity has zero entries');
assert.sameValue(Object.entries(-Infinity).length, 0, '-Infinity has zero entries');
assert.sameValue(Object.entries(NaN).length, 0, 'NaN has zero entries');
assert.sameValue(Object.entries(Math.PI).length, 0, 'Math.PI has zero entries');
