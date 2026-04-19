/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  with (...) eval(...) is a direct eval
info: bugzilla.mozilla.org/show_bug.cgi?id=601307
esid: pending
---*/

var t = "global";
function test()
{
  var t = "local";
  with ({})
    return eval("t");
}
assert.sameValue(test(), "local");
