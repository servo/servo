// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Test previous transition when nanoseconds are subtracted resp. added to the DST transition.
features: [Temporal]
---*/

const dt = Temporal.ZonedDateTime.from("2021-03-28T01:00:00Z[Europe/Berlin]");

assert.sameValue(dt.add({nanoseconds: -1}).getTimeZoneTransition("previous").toString(),
                 "2020-10-25T02:00:00+01:00[Europe/Berlin]",
                 "DST transition minus one nanosecond");

assert.sameValue(dt.getTimeZoneTransition("previous").toString(),
                 "2020-10-25T02:00:00+01:00[Europe/Berlin]",
                 "DST transition");

assert.sameValue(dt.add({nanoseconds: +1}).getTimeZoneTransition("previous").toString(),
                 "2021-03-28T03:00:00+02:00[Europe/Berlin]",
                 "DST transition plus one nanosecond");
