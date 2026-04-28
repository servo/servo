// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: Tests what information is present by default
locale: [en]
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(2024, 12, 26);
const result = plainDate.toLocaleString("en", { timeZone: "UTC" });

assert(result.includes("2024"), `PlainDate formatted with no options ${result} should include year`);
assert(result.includes("12") || result.includes("Dec"), `PlainDate formatted with no options ${result} should include month`);
assert(result.includes("26"), `PlainDate formatted with no options ${result} should include day`);
assert(!result.includes("00"), `PlainDate formatted with no options ${result} should not include hour, minute, second`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `PlainDate formatted with no options ${result} should not include time zone name`
);
