/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  __proto__ should show up with O.getOwnPropertyNames(O.prototype)
info: bugzilla.mozilla.org/show_bug.cgi?id=837630
esid: pending
---*/

var keys = Object.getOwnPropertyNames(Object.prototype);
assert.sameValue(keys.indexOf("__proto__") >= 0, true,
         "should have gotten __proto__ as a property of Object.prototype " +
         "(got these properties: " + keys + ")");
