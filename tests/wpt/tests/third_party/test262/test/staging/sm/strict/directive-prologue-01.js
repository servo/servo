/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  A string literal containing an octal escape before a strict mode directive should be a syntax error
info: bugzilla.mozilla.org/show_bug.cgi?id=601262
esid: pending
---*/

assert.throws(SyntaxError, function() {
  eval(" '\\145'; 'use strict'; ");
}, "wrong error for octal-escape before strict directive in eval");

assert.throws(SyntaxError, function() {
  Function(" '\\145'; 'use strict'; ");
}, "wrong error for octal-escape before strict directive in Function");

assert.throws(SyntaxError, function() {
  eval(" function f(){ '\\145'; 'use strict'; } ");
}, "wrong error for octal-escape before strict directive in eval of function");

assert.throws(SyntaxError, function() {
  Function(" function f(){ '\\145'; 'use strict'; } ");
}, "wrong error for octal-escape before strict directive in eval of function");

eval("function notAnError1() { 5; '\\145'; function g() { 'use strict'; } }");

Function("function notAnError2() { 5; '\\145'; function g() { 'use strict'; } }");

function notAnError3()
{
  5;
  "\145";
  function g() { "use strict"; }
}
