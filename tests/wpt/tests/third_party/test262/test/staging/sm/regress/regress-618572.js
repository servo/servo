/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Do not assert when ungetting a Unicode char sequence
info: bugzilla.mozilla.org/show_bug.cgi?id=618572
esid: pending
---*/

assert.throws(SyntaxError, function() {
  eval("var a\\0021 = 3;");
});
