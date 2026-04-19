// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Test behaviour around DST boundaries without any options set.
features: [Temporal]
---*/

// Ambiguous zoned date time - Fall DST
const DSTEnd = "2019-02-16T23:45[America/Sao_Paulo]";
let zdt = Temporal.ZonedDateTime.from(DSTEnd);
assert.sameValue(
  zdt.offset,
  "-02:00",
  "Ambiguous zoned date time");

// Non existent zoned date time - Spring DST
const DSTStart = "2020-03-08T02:30[America/Los_Angeles]";
zdt = Temporal.ZonedDateTime.from(DSTStart);
assert.sameValue(
  zdt.offset,
  "-07:00",
  "Offset result, zoned date time in non existent time");
assert.sameValue(
  zdt.hour,
  3,
  "Hour result, zoned date time in non existent time");
