/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Promote evald initializer into upvar
info: bugzilla.mozilla.org/show_bug.cgi?id=470758
esid: pending
---*/

var actual = '';
var expect = '';

test();

function test()
{
  expect = 5;

  (function(){var x;eval("for (x = 0; x < 5; x++);"); actual = x;})();

  assert.sameValue(expect, actual);
}
