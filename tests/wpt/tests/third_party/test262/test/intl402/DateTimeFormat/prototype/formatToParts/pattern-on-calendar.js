// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Checks the DateTimeFormat choose different patterns based
  on calendar.
includes: [testIntl.js]
locale: [en]
---*/

let calendars = allCalendars();
let date = new Date();

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
let base  = serializeTypesAndLiteral(df.formatToParts(date));

const foundDifferentPattern = calendars.some(function(calendar) {
  let cdf = new Intl.DateTimeFormat("en-u-ca-" + calendar);
  return base != serializeTypesAndLiteral(cdf.formatToParts(date));
});

// Expect at least some calendar use different pattern.
assert.sameValue(foundDifferentPattern, true);
