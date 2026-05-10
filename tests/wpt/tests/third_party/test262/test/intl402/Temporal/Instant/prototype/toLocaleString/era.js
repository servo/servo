// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    Instant should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const instant = new Temporal.Instant(0n);

const instantResult = instant.toLocaleString("en", { era: "narrow" });
const dateResult = new Date(0).toLocaleString(["en"], { era: "narrow" });
assert.sameValue(instantResult, dateResult, "toLocaleString on an Instant with era option should return the same results as toLocaleString on the same Date with the same options");
