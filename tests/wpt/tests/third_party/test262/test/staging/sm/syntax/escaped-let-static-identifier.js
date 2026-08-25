/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  |let| and |static| are forbidden as Identifier only in strict mode code, and it's permissible to use them as Identifier (with or without containing escapes) in non-strict code
info: bugzilla.mozilla.org/show_bug.cgi?id=1288460
esid: pending
---*/

function t(code)
{
  var strictSemi = " 'use strict'; " + code;
  var strictASI = " 'use strict' \n " + code;

  Function(code);

  assert.throws(SyntaxError, () => Function(strictSemi));
  assert.throws(SyntaxError, () => Function(strictASI));
}

t("l\\u0065t: 42;");
t("if (1) l\\u0065t: 42;");
t("l\\u0065t = 42;");
t("if (1) l\\u0065t = 42;");

t("st\\u0061tic: 42;");
t("if (1) st\\u0061tic: 42;");
t("st\\u0061tic = 42;");
t("if (1) st\\u0061tic = 42;");
