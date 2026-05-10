// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Fallback value for offset option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloffset step 1:
      1. Return ? GetOption(_normalizedOptions_, *"offset"*, « String », « *"prefer"*, *"use"*, *"ignore"*, *"reject"* », _fallback_).
    sec-temporal.zoneddatetime.protoype.with step 15:
      15. Let _offset_ be ? ToTemporalOffset(_options_, *"prefer"*).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1572757201_000_000_000n, "-03:30");
const explicit = datetime.with({ minute: 31 }, { offset: undefined });
assert.sameValue(explicit.epochNanoseconds, 1572757261_000_000_000n, "default offset is prefer");
const implicit = datetime.with({ minute: 31 }, {});
assert.sameValue(implicit.epochNanoseconds, 1572757261_000_000_000n, "default offset is prefer");
const lambda = datetime.with({ minute: 31 }, () => {});
assert.sameValue(lambda.epochNanoseconds, 1572757261_000_000_000n, "default offset is prefer");
