// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: >
  Accessor property for "timeZone" throws an error.
info: |
  Temporal.Instant.prototype.toString ( [ options ] )

  ...
  9. Let timeZone be ? Get(resolvedOptions, "timeZone").
  ...
features: [Temporal]
---*/

var instance = new Temporal.Instant(0n);

var options = {
  get timeZone() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => instance.toString(options));
