// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    PlainTime should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const date = new Temporal.PlainTime(14, 46);

const result = date.toLocaleString("en", { era: "narrow" });

assert(result.startsWith("2"), "toLocaleString on a PlainTime with era option should work");
assert(!result.includes("A"), "era should be ignored when formatting a PlainTime");
