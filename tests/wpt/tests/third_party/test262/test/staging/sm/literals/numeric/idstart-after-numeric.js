/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  numeric literal followed by an identifier
info: bugzilla.mozilla.org/show_bug.cgi?id='523401'
esid: pending
---*/

var array = new Array();
assert.throws(SyntaxError, () => eval("array[0for]"));
assert.throws(SyntaxError, () => eval("array[1yield]"));
assert.throws(SyntaxError, () => eval("array[2in []]")); // "2 in []" is valid.
assert.sameValue(array[2 in []], undefined);
assert.sameValue(2 in [], false);
assert.throws(SyntaxError, () => eval("array[3in]"));
