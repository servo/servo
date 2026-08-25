/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.match behavior with zero-length matches involving forward lookahead
info: bugzilla.mozilla.org/show_bug.cgi?id=501739
esid: pending
---*/

var r = /(?=x)/g;

var res = "aaaaaaaaaxaaaaaaaaax".match(r);
assert.sameValue(res.length, 2);
