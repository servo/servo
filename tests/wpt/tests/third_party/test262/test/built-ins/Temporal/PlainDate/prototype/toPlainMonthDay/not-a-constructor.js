// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplainmonthday
description: >
  Temporal.PlainDate.prototype.toPlainMonthDay does not implement [[Construct]], is not new-able
info: |
    Built-in function objects that are not identified as constructors do not implement the
    [[Construct]] internal method unless otherwise specified in the description of a particular
    function.
includes: [isConstructor.js]
features: [Reflect.construct, Temporal]
---*/

assert.throws(TypeError, () => {
  new Temporal.PlainDate.prototype.toPlainMonthDay();
}, "Calling as constructor");

assert.sameValue(isConstructor(Temporal.PlainDate.prototype.toPlainMonthDay), false,
  "isConstructor(Temporal.PlainDate.prototype.toPlainMonthDay)");
