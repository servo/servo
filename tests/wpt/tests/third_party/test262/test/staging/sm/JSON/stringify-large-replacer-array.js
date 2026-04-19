/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JSON.stringify with a large replacer array
info: bugzilla.mozilla.org/show_bug.cgi?id=816033
esid: pending
---*/

var replacer = [];
for (var i = 0; i < 4096; i++)
  replacer.push(i);

assert.sameValue(JSON.stringify({ "foopy": "FAIL", "4093": 17 }, replacer), '{"4093":17}');
