/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  |var eval = otherWindow.eval; eval(...)| should behave like indirectly calling that eval from a script in that other window
info: bugzilla.mozilla.org/show_bug.cgi?id=608473
esid: pending
---*/

var originalEval = eval;
var res;

function f()
{
  return [this, eval("this")];
}

var otherGlobal = $262.createRealm().global;

eval = otherGlobal.eval;
res = new f();
assert.sameValue(res[0] !== res[1], true);
assert.sameValue(res[0] !== this, true);
assert.sameValue(res[0] instanceof f, true);
assert.sameValue(res[1], otherGlobal);

res = f();
assert.sameValue(res[0] !== res[1], true);
assert.sameValue(res[0], this);
assert.sameValue(res[1], otherGlobal);
