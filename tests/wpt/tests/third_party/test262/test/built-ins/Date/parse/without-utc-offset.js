// Copyright (C) 2020 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.parse
description: >
  Offsetless date-time strings are local time, offsetless date-only strings are UTC time
info: |
  Date.parse ( string )

  When the UTC offset representation is absent, date-only forms are interpreted
  as a UTC time and date-time forms are interpreted as a local time.
---*/

const timezoneOffsetMS = new Date(0).getTimezoneOffset() * 60000;

assert.sameValue(Date.parse('1970-01-01T00:00:00'), timezoneOffsetMS);
assert.sameValue(Date.parse('1970-01-01'), 0);
