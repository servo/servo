/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.getOwnPropertyNames: regular expression objects
info: bugzilla.mozilla.org/show_bug.cgi?id=518663
esid: pending
---*/

var actual = Object.getOwnPropertyNames(/a/);
var expected = ["lastIndex"];

for (var i = 0; i < expected.length; i++)
{
  assert.sameValue(actual.indexOf(expected[i]) >= 0, true,
                expected[i] + " should be a property name on a RegExp");
}
