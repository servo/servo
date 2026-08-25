/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Fractional days, months, years shouldn't trigger asserts
info: bugzilla.mozilla.org/show_bug.cgi?id=771946
esid: pending
---*/

new Date(0).setFullYear(1.5);
new Date(0).setUTCDate(1.5);
new Date(0).setMonth(1.5);
