// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Constructors can't be called so we can't pattern match
// them in replace and sort.
function a() {
    var b = {a: "A"};

    class X {
        constructor(a) {
            return b[a]
        }
    };

    assert.throws(TypeError, () => "a".replace(/a/, X));
}

function b() {
    class X {
        constructor(x, y) {
            return x - y;
        }
    }

    assert.throws(TypeError, () => [1, 2, 3].sort(X));
}

a();
b();

