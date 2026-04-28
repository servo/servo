/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Number.parseInt(string, radix). Verify that Number.parseInt defaults to decimal.
info: bugzilla.mozilla.org/show_bug.cgi?id=886949
esid: pending
---*/

assert.sameValue(Number.parseInt("08"), 8);
assert.sameValue(Number.parseInt("09"), 9);
assert.sameValue(Number.parseInt("014"), 14);

function strictParseInt(s) { "use strict"; return Number.parseInt(s); }

assert.sameValue(strictParseInt("08"), 8);
assert.sameValue(strictParseInt("09"), 9);
assert.sameValue(strictParseInt("014"), 14);
