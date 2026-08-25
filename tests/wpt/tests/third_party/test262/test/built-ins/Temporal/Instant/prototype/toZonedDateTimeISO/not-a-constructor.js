// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: >
  Temporal.Instant.prototype.toZonedDateTimeISO does not implement [[Construct]], is not new-able
info: |
    Built-in function objects that are not identified as constructors do not implement the
    [[Construct]] internal method unless otherwise specified in the description of a particular
    function.
includes: [isConstructor.js]
features: [Reflect.construct, Temporal]
---*/

assert.throws(TypeError, () => {
  new Temporal.Instant.prototype.toZonedDateTimeISO();
}, "Calling as constructor");

assert.sameValue(isConstructor(Temporal.Instant.prototype.toZonedDateTimeISO), false,
  "isConstructor(Temporal.Instant.prototype.toZonedDateTimeISO)");
