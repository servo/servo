// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Test that "Z" and a timezone in a string argument preserves the exact time in
  the given time zone.
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2020-03-08T09:00:00Z[America/Los_Angeles]");
assert.sameValue(zdt.hour, 1, "hour");
assert.sameValue(zdt.minute, 0, "minute");
assert.sameValue(zdt.second, 0, "second");
assert.sameValue(zdt.offset, "-08:00", "offset");
