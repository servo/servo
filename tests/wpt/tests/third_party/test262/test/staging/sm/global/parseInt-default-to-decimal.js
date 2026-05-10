/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  parseInt should treat leading-zero inputs (with radix unspecified) as decimal, not octal
info: bugzilla.mozilla.org/show_bug.cgi?id=583925
esid: pending
---*/

assert.sameValue(parseInt("08"), 8);
assert.sameValue(parseInt("09"), 9);
assert.sameValue(parseInt("014"), 14);

function strictParseInt(s) { "use strict"; return parseInt(s); }

assert.sameValue(strictParseInt("08"), 8);
assert.sameValue(strictParseInt("09"), 9);
assert.sameValue(strictParseInt("014"), 14);
