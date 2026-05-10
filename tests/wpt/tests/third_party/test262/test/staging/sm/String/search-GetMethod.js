// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.search should call GetMethod.
info: bugzilla.mozilla.org/show_bug.cgi?id=1290655
esid: pending
---*/

function create(value) {
    return {
        [Symbol.search]: value,
        toString() {
            return "-";
        }
    };
}

for (let v of [null, undefined]) {
    assert.sameValue("a-a".search(create(v)), 1);
}

for (let v of [1, true, Symbol.iterator, "", {}, []]) {
    assert.throws(TypeError, () => "a-a".search(create(v)));
}
