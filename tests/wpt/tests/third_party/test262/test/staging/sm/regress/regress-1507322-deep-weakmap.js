// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
features: [host-gc-required]
---*/
function TestGC2(m) {
  var head = new Object;
  for (var key = head, i = 0; i < 99999; i++, key = m.get(key)) {
    m.set(key, new Object);
  }
  $262.gc();
  for (key = head; key != undefined; key = m.get(key)) {}
}
TestGC2(new WeakMap);

assert.sameValue(true, true, "deep weakmaps");
