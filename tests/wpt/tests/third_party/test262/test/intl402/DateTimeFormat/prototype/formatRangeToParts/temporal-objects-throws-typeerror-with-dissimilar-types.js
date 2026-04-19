// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Throw TypeError when called with dissimilar types
features: [Temporal]
---*/

const dtf = new Intl.DateTimeFormat();
const t1 = "1976-11-18T14:23:30+00:00[UTC]";
const t2 = "2020-02-20T15:44:56-05:00[America/New_York]";

assert.throws(TypeError, () =>
  dtf.formatRangeToParts(Temporal.Instant.from(t1), Temporal.PlainDateTime.from(t2))
);
