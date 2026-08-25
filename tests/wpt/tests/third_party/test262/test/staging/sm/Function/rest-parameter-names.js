/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Rest parameters to functions can be named |yield| or |eval| or |let| in non-strict code
info: bugzilla.mozilla.org/show_bug.cgi?id=1288460
esid: pending
---*/

var f1 = (...yield) => yield + 42;
assert.sameValue(f1(), "42");
assert.sameValue(f1(1), "142");

var f2 = (...eval) => eval + 42;
assert.sameValue(f2(), "42");
assert.sameValue(f2(1), "142");

var f3 = (...let) => let + 42;
assert.sameValue(f3(), "42");
assert.sameValue(f3(1), "142");

function g1(x, ...yield)
{
  return yield + x;
}
assert.sameValue(g1(0, 42), "420");

function g2(x, ...eval)
{
  return eval + x;
}
assert.sameValue(g2(0, 42), "420");

function g3(x, ...let)
{
  return let + x;
}
assert.sameValue(g3(0, 42), "420");

function h()
{
  "use strict";

  var badNames = ["yield", "eval", "let"];

  for (var badName of ["yield", "eval", "let"])
  {
    assert.throws(SyntaxError, () => eval(`var q = (...${badName}) => ${badName} + 42;`));

    assert.throws(SyntaxError, () => eval(`function r(x, ...${badName}) { return x + ${badName}; }`));
  }
}
h();
