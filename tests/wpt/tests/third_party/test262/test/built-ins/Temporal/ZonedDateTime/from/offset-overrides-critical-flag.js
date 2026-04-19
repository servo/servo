// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  The offset option always overrides the critical flag in a time zone annotation
features: [Temporal]
---*/

const useResult = Temporal.ZonedDateTime.from("2022-10-07T18:37-07:00[!UTC]", { offset: "use" });
assert.sameValue(
  useResult.epochNanoseconds,
  1665193020000000000n,
  "exact time is unchanged with offset = use, despite critical flag"
);

const ignoreResult = Temporal.ZonedDateTime.from("2022-10-07T18:37-07:00[!UTC]", { offset: "ignore" });
assert.sameValue(
  ignoreResult.epochNanoseconds,
  1665167820000000000n,
  "wall time is unchanged with offset = ignore, despite critical flag"
);

const preferResult = Temporal.ZonedDateTime.from("2022-10-07T18:37-07:00[!UTC]", { offset: "prefer" });
assert.sameValue(
  useResult.epochNanoseconds,
  1665193020000000000n,
  "offset is recalculated with offset = prefer, despite critical flag"
);
