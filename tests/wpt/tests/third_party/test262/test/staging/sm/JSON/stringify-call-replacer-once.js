/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Call replacer function exactly once per value
info: bugzilla.mozilla.org/show_bug.cgi?id=584909
esid: pending
---*/

var factor = 1;
function replacer(k, v)
{
  if (k === "")
    return v;

  return v * ++factor;
}

var obj = { a: 1, b: 2, c: 3 };

assert.sameValue(JSON.stringify(obj, replacer), '{"a":2,"b":6,"c":12}');
assert.sameValue(factor, 4);
