// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries accepts Symbol primitives.
author: Jordan Harband
features: [Symbol]
---*/

var result = Object.entries(Symbol());

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 0, 'result has 0 items');
