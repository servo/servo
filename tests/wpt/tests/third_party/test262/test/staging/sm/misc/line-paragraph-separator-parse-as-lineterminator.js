/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  U+2028 LINE SEPARATOR and U+2029 PARAGRAPH SEPARATOR must match the LineTerminator production when parsing code
info: bugzilla.mozilla.org/show_bug.cgi?id=663331
esid: pending
---*/

var hidden = 17;
var assigned;

assigned = 42;
assert.sameValue(eval('"use strict"; var hidden\u2028assigned = 5; typeof hidden'),
         "undefined");
assert.sameValue(assigned, 5);

assigned = 42;
function t1()
{
  assert.sameValue(eval('var hidden\u2028assigned = 5; typeof hidden'), "undefined");
  assert.sameValue(assigned, 5);
}
t1();

assigned = 42;
assert.sameValue(eval('"use strict"; var hidden\u2029assigned = 5; typeof hidden'),
         "undefined");
assert.sameValue(assigned, 5);

assigned = 42;
function t2()
{
  assert.sameValue(eval('var hidden\u2029assigned = 5; typeof hidden'), "undefined");
  assert.sameValue(assigned, 5);
}
t2();
