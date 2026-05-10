/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
features: [host-gc-required]
---*/
var x = {}, h = new WeakMap;
h.set(x, null);
$262.gc();

