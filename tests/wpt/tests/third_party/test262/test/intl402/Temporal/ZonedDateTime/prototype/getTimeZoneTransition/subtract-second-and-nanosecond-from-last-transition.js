// Copyright (C) 2022 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.gettimezonetransition
description: >
  Compute next transition when seconds resp. nanoseconds are subtracted from the last transition.
features: [Temporal]
---*/

// From <https://github.com/eggert/tz/blob/main/europe>:
//
// # Zone  NAME       STDOFF  RULES FORMAT  [UNTIL]
// Zone  Europe/Paris 0:09:21 -     LMT     1891 Mar 15  0:01
//                    0:09:21 -     PMT     1911 Mar 11  0:01 # Paris MT

const zdt = new Temporal.PlainDateTime(1800, 1, 1).toZonedDateTime("Europe/Paris");
assert.sameValue(zdt.toString(), "1800-01-01T00:00:00+00:09[Europe/Paris]");
assert.sameValue(zdt.offsetNanoseconds, (9 * 60 + 21) * 1_000_000_000);

// Ensure the first transition was correctly computed.
const first = zdt.getTimeZoneTransition("next");
assert.sameValue(first.toString(), "1911-03-10T23:50:39+00:00[Europe/Paris]");

let next;

// Compute the next transition starting from the first transition minus 1s.
const firstMinus1s = first.add({seconds: -1});
assert.sameValue(firstMinus1s.toString(), "1911-03-10T23:59:59+00:09[Europe/Paris]");
assert.sameValue(firstMinus1s.offsetNanoseconds, (9 * 60 + 21) * 1_000_000_000);

next = firstMinus1s.getTimeZoneTransition("next");
assert.sameValue(next.toString(), "1911-03-10T23:50:39+00:00[Europe/Paris]");

// Compute the next transition starting from the first transition minus 1ns.
const firstMinus1ns = first.add({nanoseconds: -1});
assert.sameValue(firstMinus1ns.toString(), "1911-03-10T23:59:59.999999999+00:09[Europe/Paris]");
assert.sameValue(firstMinus1ns.offsetNanoseconds, (9 * 60 + 21) * 1_000_000_000);

next = firstMinus1ns.getTimeZoneTransition("next");
assert.sameValue(next.toString(), "1911-03-10T23:50:39+00:00[Europe/Paris]");
