// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: Temporal.Duration.prototype.toJSON does not support options, unlike toString.
features: [Temporal]
---*/

let called = 0;
const options = new Proxy({}, {
  get() {
    ++called;
  }
});
const d = new Temporal.Duration(1, 2);
assert.sameValue(d.toJSON(options), "P1Y2M");
assert.sameValue(called, 0);
