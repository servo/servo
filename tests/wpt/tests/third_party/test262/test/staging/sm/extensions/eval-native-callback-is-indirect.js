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

function testArrayGeneric()
{
  var global = "psych!";
  var eval = Array.map;

  var mapped = eval([directCheckCode], originalEval);
  assert.sameValue(mapped[0], true);
}
