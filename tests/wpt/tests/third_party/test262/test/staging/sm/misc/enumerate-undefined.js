/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JS_EnumerateStandardClasses uses wrong attributes for undefined
info: bugzilla.mozilla.org/show_bug.cgi?id=547087
esid: pending
---*/

for (var p in this);

assert.sameValue(Object.getOwnPropertyDescriptor(this, "undefined").writable, false);
