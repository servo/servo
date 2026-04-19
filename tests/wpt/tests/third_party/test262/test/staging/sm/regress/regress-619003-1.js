/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

var a = [];

// Test up to 200 to cover tunables such as js::PropertyTree::MAX_HEIGHT.
for (var i = 0; i < 200; i++) {
    a.push("b" + i);
    assert.throws(
        SyntaxError,
        () => eval("(function ([" + a.join("],[") + "],a,a){})"),
        'duplicate argument names not allowed in this context');
}
