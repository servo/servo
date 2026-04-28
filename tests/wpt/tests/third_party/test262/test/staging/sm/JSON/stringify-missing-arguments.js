/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JSON.stringify with no arguments
info: bugzilla.mozilla.org/show_bug.cgi?id=648471
esid: pending
---*/

assert.sameValue(JSON.stringify(), undefined);
