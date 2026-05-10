// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Offset string time zones compare as expected
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "America/Los_Angeles");
assert(zdt.withTimeZone("+05:30").equals(zdt.withTimeZone("+0530")), "+05:30 = +0530");
assert(zdt.withTimeZone("+0530").equals(zdt.withTimeZone("+05:30")), "+0530 = +05:30");
assert(zdt.withTimeZone("+05:30").equals(zdt.withTimeZone("+0530").toString()), "+05:30 = +0530 IXDTF string");
assert(!zdt.withTimeZone("+05:30").equals(zdt.withTimeZone("Asia/Kolkata")), "+05:30 != Asia/Kolkata string ID");
assert(!zdt.withTimeZone("+05:30").equals(zdt.withTimeZone("Asia/Kolkata").toString()), "+05:30 != Asia/Kolkata IXDTF string");
