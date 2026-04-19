// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Checks the DateTimeFormat choose different patterns based
  on calendar.
includes: [testIntl.js]
features: [Intl.DateTimeFormat-formatRange]
locale: [en]
---*/

let calendars = allCalendars();
let date1 = new Date(2017, 3, 12);
let date2 = new Date();

// serialize parts to a string by considering only the type and literal.
function serializeTypesAndLiteral(parts) {
  let types = parts.map(part => {
    if (part.type == "literal") {
      return `${part.type}(${part.value})`;
    }
    return part.type;
  });
  return types.join(":");
}

let df = new Intl.DateTimeFormat("en");
let base  = serializeTypesAndLiteral(df.formatRangeToParts(date1, date2));

const foundDifferentPattern = calendars.some(function(calendar) {
  let cdf = new Intl.DateTimeFormat("en-u-ca-" + calendar);
  return base != serializeTypesAndLiteral(cdf.formatRangeToParts(date1, date2));
});

// Expect at least some calendar use different pattern.
assert.sameValue(foundDifferentPattern, true);
