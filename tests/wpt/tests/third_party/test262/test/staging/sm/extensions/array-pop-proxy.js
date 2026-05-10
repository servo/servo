/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Behavior of [].pop on proxies
info: bugzilla.mozilla.org/show_bug.cgi?id=858381
esid: pending
---*/

var p = new Proxy([0, 1, 2], {});
Array.prototype.pop.call(p);
