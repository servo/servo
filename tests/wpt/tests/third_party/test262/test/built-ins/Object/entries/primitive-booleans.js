// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries accepts boolean primitives.
author: Jordan Harband
---*/

var trueResult = Object.entries(true);

assert.sameValue(Array.isArray(trueResult), true, 'trueResult is an array');
assert.sameValue(trueResult.length, 0, 'trueResult has 0 items');

var falseResult = Object.entries(false);

assert.sameValue(Array.isArray(falseResult), true, 'falseResult is an array');
assert.sameValue(falseResult.length, 0, 'falseResult has 0 items');
