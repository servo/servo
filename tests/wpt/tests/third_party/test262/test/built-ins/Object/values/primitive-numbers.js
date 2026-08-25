// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: Object.values accepts number primitives.
author: Jordan Harband
---*/

assert.sameValue(Object.values(0).length, 0, '0 has zero values');
assert.sameValue(Object.values(-0).length, 0, '-0 has zero values');
assert.sameValue(Object.values(Infinity).length, 0, 'Infinity has zero values');
assert.sameValue(Object.values(-Infinity).length, 0, '-Infinity has zero values');
assert.sameValue(Object.values(NaN).length, 0, 'NaN has zero values');
assert.sameValue(Object.values(Math.PI).length, 0, 'Math.PI has zero values');
