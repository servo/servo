/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  delete (foo), delete ((foo)), and so on are strict mode early errors
info: bugzilla.mozilla.org/show_bug.cgi?id=1111101
esid: pending
---*/

function checkSyntaxError(code)
{
  assert.throws(SyntaxError, function() {
    Function(code);
  });
  assert.throws(SyntaxError, function() {
    (1, eval)(code); // indirect eval
  });
}

checkSyntaxError("function f() { 'use strict'; delete escape; } f();");
checkSyntaxError("function f() { 'use strict'; delete escape; }");
checkSyntaxError("function f() { 'use strict'; delete (escape); } f();");
checkSyntaxError("function f() { 'use strict'; delete (escape); }");
checkSyntaxError("function f() { 'use strict'; delete ((escape)); } f();");
checkSyntaxError("function f() { 'use strict'; delete ((escape)); }");

// Meanwhile, non-strict all of these should work

function checkFine(code)
{
  Function(code);
  (1, eval)(code); // indirect, to be consistent w/above
}

checkFine("function f() { delete escape; } f();");
checkFine("function f() { delete escape; }");
checkFine("function f() { delete (escape); } f();");
checkFine("function f() { delete (escape); }");
checkFine("function f() { delete ((escape)); } f();");
checkFine("function f() { delete ((escape)); }");
