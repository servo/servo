// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.hypot
es6id: 20.2.2.18
author: Ryan Lewis
description: Math.hypot should return 0 if all arguments are 0 or -0.
---*/

assert.sameValue(Math.hypot(0), 0, 'Math.hypot(0)');
assert.sameValue(Math.hypot(-0), 0, 'Math.hypot(-0)');
assert.sameValue(Math.hypot(0, 0), 0, 'Math.hypot(0, 0)');
assert.sameValue(Math.hypot(0, -0), 0, 'Math.hypot(0, -0)');
assert.sameValue(Math.hypot(-0, 0), 0, 'Math.hypot(-0, 0)');
assert.sameValue(Math.hypot(-0, -0), 0, 'Math.hypot(-0, -0)');
assert.sameValue(Math.hypot(0, -0, -0), 0, 'Math.hypot(0, -0, -0)');
