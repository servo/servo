// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    PlainMonthDay should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const date = new Temporal.PlainMonthDay(5, 2, "gregory");

assert(date.toLocaleString("en", { era: "narrow" }).startsWith("5"), "toLocaleString on a PlainMonthDay with era option should work");
