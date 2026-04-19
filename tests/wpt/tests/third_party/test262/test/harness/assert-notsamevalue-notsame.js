// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Values that are not strictly equal satisfy the assertion.
---*/

assert.notSameValue(undefined, null);
assert.notSameValue(null, undefined);
assert.notSameValue(0, 1);
assert.notSameValue(1, 0);
assert.notSameValue('', 's');
assert.notSameValue('s', '');
