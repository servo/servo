/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Support initializer defaults in destructuring declarations in for-in/of loop heads
info: bugzilla.mozilla.org/show_bug.cgi?id=1233767
esid: pending
---*/

var count;
var expr;

expr = [{ z: 42, 42: "hi" }, { 7: 'fnord' }];
count = 0;
for (var { z: x = 7, [x]: y = 3 } of expr)
{
  if (count === 0) {
    assert.sameValue(x, 42);
    assert.sameValue(y, "hi");
  } else {
    assert.sameValue(x, 7);
    assert.sameValue(y, "fnord");
  }

  count++;
}

count = 0;
for (var { length: x, [x - 1 + count]: y = "psych" } in "foo")
{
  assert.sameValue(x, 1);
  assert.sameValue(y, count === 0 ? "0" : "psych");

  count++;
}
