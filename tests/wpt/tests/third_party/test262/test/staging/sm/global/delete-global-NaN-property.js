/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - onlyStrict
description: |
  |delete window.NaN| should throw a TypeError
info: bugzilla.mozilla.org/show_bug.cgi?id=649570
esid: pending
---*/

var g = this, v = false;
try
{
  delete this.NaN;
  throw new Error("no exception thrown");
}
catch (e)
{
  assert.sameValue(e instanceof TypeError, true,
           "Expected a TypeError, got: " + e);
}
