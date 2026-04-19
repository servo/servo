/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't assert parsing |for (var x = 3 of 42);|
info: bugzilla.mozilla.org/show_bug.cgi?id=1164741
esid: pending
---*/

assert.throws(SyntaxError, function() {
  Function("for (var x = 3 of 42);");
});
