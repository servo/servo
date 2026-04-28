/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - onlyStrict
description: |
  Verify that we don't optimize free names to gnames in eval code that's global, when the name refers to a binding introduced by a strict mode eval frame
info: bugzilla.mozilla.org/show_bug.cgi?id=514568
esid: pending
---*/

var x = "global";
assert.sameValue(eval('var x = "eval"; eval("x")'), "eval");
