/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  '0x' not followed by hex digits should be a syntax error
info: bugzilla.mozilla.org/show_bug.cgi?id=582643
esid: pending
---*/

assert.throws(SyntaxError, function() {
  eval("0x");
});
