// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: String argument creates options object with null prototype, so Object.prototype pollution doesn't affect it
features: [Temporal]
---*/

const props = ["relativeTo", "roundingMode", "roundingIncrement", "largestUnit"];
for (const prop of props) {
  Object.defineProperty(Object.prototype, prop, {
    get() {
      throw new Test262Error(`Object.prototype.${prop} was looked up`);
    },
    configurable: true,
  });
}

try {
  const instance = new Temporal.Duration(0, 0, 0, 0, 1, 30);
  instance.round("hour");
} finally {
  for (const prop of props) {
    delete Object.prototype[prop];
  }
}
