/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
info: |
  preventExtensions on global
  bugzilla.mozilla.org/show_bug.cgi?id=621432
description: |
  If a var statement can't create a global property because the global object isn't extensible, and an error is thrown while decompiling the global, don't assert
esid: pending
---*/

var toSource = [];
Object.preventExtensions(this);

try
{
  eval("var x;");
  throw new Error("no error thrown");
}
catch (e)
{
  assert.sameValue(e instanceof TypeError, true, "expected TypeError, got: " + e);
}
