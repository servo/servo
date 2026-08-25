// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: String argument creates options object with null prototype, so Object.prototype pollution doesn't affect it
features: [Temporal]
---*/

const props = ["roundingIncrement", "roundingMode"];
for (const prop of props) {
  Object.defineProperty(Object.prototype, prop, {
    get() {
      throw new Test262Error(`Object.prototype.${prop} was looked up`);
    },
    configurable: true,
  });
}

try {
  const instance = new Temporal.Instant(1_000_000_000_123_456_789n);
  instance.round("hour");
} finally {
  for (const prop of props) {
    delete Object.prototype[prop];
  }
}
