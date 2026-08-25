/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  new Date(value) should call ToPrimitive on value before testing for string-ness
info: bugzilla.mozilla.org/show_bug.cgi?id=738511
esid: pending
---*/

assert.sameValue(new Date(new String("2012-01-31T00:00:00.000Z")).valueOf(),
         1327968000000,
         "Date constructor passed a String object should parse it");
