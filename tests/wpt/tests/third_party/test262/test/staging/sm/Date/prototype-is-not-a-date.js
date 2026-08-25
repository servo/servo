/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Date.prototype isn't an instance of Date
info: bugzilla.mozilla.org/show_bug.cgi?id=861219
esid: pending
---*/

assert.sameValue(Date.prototype instanceof Date, false);
assert.sameValue(Date.prototype.__proto__, Object.prototype);
