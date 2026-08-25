// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    PlainYearMonth should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const date = new Temporal.PlainYearMonth(2000, 5, "gregory");

assert(date.toLocaleString("en", { era: "narrow" }).startsWith("5"), "toLocaleString on a PlainYearMonth with era option should work");
