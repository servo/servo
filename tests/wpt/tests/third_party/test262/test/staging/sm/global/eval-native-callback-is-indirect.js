/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  eval called from a native function is indirect
info: bugzilla.mozilla.org/show_bug.cgi?id=604504
esid: pending
---*/

var originalEval = eval;

var global = this;
var directCheckCode = "this === global";

function testBound()
{
  var global = "psych!";
  var eval = originalEval.bind(undefined, directCheckCode);
  assert.sameValue(eval(), true);
}
testBound();
