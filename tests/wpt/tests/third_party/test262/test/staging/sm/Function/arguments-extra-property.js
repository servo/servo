// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  GetElem for modified arguments shouldn't be optimized to get original argument.
info: bugzilla.mozilla.org/show_bug.cgi?id=1263811
esid: pending
---*/

function testModifyFirst() {
    function f() {
        Object.defineProperty(arguments, 1, { value: 30 });
        assert.sameValue(arguments[1], 30);
    }
    for (let i = 0; i < 10; i++)
        f(10, 20);
}
testModifyFirst();

function testModifyLater() {
    function f() {
        var ans = 20;
        for (let i = 0; i < 10; i++) {
            if (i == 5) {
                Object.defineProperty(arguments, 1, { value: 30 });
                ans = 30;
            }
            assert.sameValue(arguments[1], ans);
        }
    }
    for (let i = 0; i < 10; i++)
        f(10, 20);
}
testModifyLater();
