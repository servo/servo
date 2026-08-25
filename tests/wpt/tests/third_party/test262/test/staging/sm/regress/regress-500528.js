/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// This test appeared in bug 497789 comment 78.

var a = {x: 'a'},
    b1 = Object.create(a),
    c1 = Object.create(b1),
    b2 = Object.create(a),
    c2 = Object.create(b2);

b2.x = 'b';  // foreshadowing a.x

var s = '';
for (var obj of [c1, c2])
    s += obj.x;
assert.sameValue(s, 'ab');

assert.sameValue(0, 0, "Property cache soundness: objects with the same shape but different prototypes.");
