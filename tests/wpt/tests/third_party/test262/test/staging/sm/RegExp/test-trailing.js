// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Trailing .* should not be ignored on matchOnly match.
info: bugzilla.mozilla.org/show_bug.cgi?id=1304737
esid: pending
---*/

function test(r, lastIndexIsZero) {
    r.lastIndex = 0;
    r.test("foo");
    assert.sameValue(r.lastIndex, lastIndexIsZero ? 0 : 3);

    r.lastIndex = 0;
    r.test("foo\nbar");
    assert.sameValue(r.lastIndex, lastIndexIsZero ? 0 : 3);

    var input = "foo" + ".bar".repeat(20000);
    r.lastIndex = 0;
    r.test(input);
    assert.sameValue(r.lastIndex, lastIndexIsZero ? 0 : input.length);

    r.lastIndex = 0;
    r.test(input + "\nbaz");
    assert.sameValue(r.lastIndex, lastIndexIsZero ? 0 : input.length);
}

test(/f.*/, true);
test(/f.*/g, false);
test(/f.*/y, false);
test(/f.*/gy, false);
