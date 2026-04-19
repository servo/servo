// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function f(a = eval(`
    function g() {
        'use strict';
        return this;
    }

    with ({}) {
        g() /* implicit return value */
    }
    `)) {
    return a
};

assert.sameValue(f(), undefined);

