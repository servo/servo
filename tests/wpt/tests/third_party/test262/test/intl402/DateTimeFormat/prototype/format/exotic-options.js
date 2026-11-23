// Copyright (C) 2026 Adam Shaw. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Inherited and non-enumerable options apply
locale: [en]
---*/

const date = new Date(Date.UTC(2020, 4, 15));

// Baseline, using ordinary own enumerable properties.
const expected = new Intl.DateTimeFormat("en", { month: "long", day: "numeric" }).format(date);

function assertMonthLongDayNumeric(options, description) {
  const formatter = new Intl.DateTimeFormat("en", options);
  const resolved = formatter.resolvedOptions();

  assert.sameValue(resolved.month, "long", `${description}: month option`);
  assert.sameValue(resolved.day, "numeric", `${description}: day option`);
  assert.sameValue(formatter.format(date), expected, `${description}: formatting a PlainDate`);
}

// Options living on the prototype chain instead of being own properties.
assertMonthLongDayNumeric(
  Object.create({ month: "long", day: "numeric" }),
  "inherited options"
);

// Own properties that are not enumerable.
assertMonthLongDayNumeric(
  Object.defineProperties({}, {
    month: { value: "long", enumerable: false },
    day: { value: "numeric", enumerable: false },
  }),
  "non-enumerable options"
);
