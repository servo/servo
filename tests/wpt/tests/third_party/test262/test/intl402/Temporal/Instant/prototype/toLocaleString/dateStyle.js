// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Basic tests that dateStyle option affects output
locale: [en-u-ca-gregory, en-u-ca-islamic-tbla]
features: [Temporal, Intl.DateTimeFormat-datetimestyle]
---*/

const instant = new Temporal.Instant(1711475200_000_000_000n);

assert(
  instant.toLocaleString("en-u-ca-gregory", { dateStyle: "long" }).includes("March"),
  "dateStyle: long writes month of March out in full"
);
assert(
  !instant.toLocaleString("en-u-ca-gregory", { dateStyle: "short" }).includes("March"),
  "dateStyle: short does not write month of March out in full"
);
assert(
  instant.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "long" }).includes("Ramadan"),
  "dateStyle: long writes month of Ramadan out in full"
);
assert(
  !instant.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "short" }).includes("Ramadan"),
  "dateStyle: short does not write month of Ramadan out in full"
);
