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
for (var u = 0; u < 3; ++u) {
    var y = [];
    Object.create(y);
    $262.gc();
    y.t = 3;
    $262.gc();
}

