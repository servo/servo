/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.indexOf with empty searchString
info: bugzilla.mozilla.org/show_bug.cgi?id=612838
esid: pending
---*/

assert.sameValue("123".indexOf("", -1), 0);
assert.sameValue("123".indexOf("", 0), 0);
assert.sameValue("123".indexOf("", 1), 1);
assert.sameValue("123".indexOf("", 3), 3);
assert.sameValue("123".indexOf("", 4), 3);
assert.sameValue("123".indexOf("", 12345), 3);
