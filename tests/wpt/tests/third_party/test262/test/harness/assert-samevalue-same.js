// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Values that are strictly equal satisfy the assertion.
---*/
var obj;

assert.sameValue(undefined, undefined);
assert.sameValue(null, null);
assert.sameValue(0, 0);
assert.sameValue(1, 1);
assert.sameValue('', '');
assert.sameValue('s', 's');

obj = {};
assert.sameValue(obj, obj);
