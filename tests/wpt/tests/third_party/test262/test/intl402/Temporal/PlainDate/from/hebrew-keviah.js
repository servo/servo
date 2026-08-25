// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Check that computed kevi'ah symbols for years are in the valid set
features: [Temporal, Intl.Era-monthcode]
---*/

// Ensure kevi'ah is correct.
//
// https://en.wikipedia.org/wiki/Hebrew_calendar#Keviah

function KeviahSymbol(year) {
  let startOfYear = Temporal.PlainDate.from({
    calendar: "hebrew",
    year,
    monthCode: "M01",
    day: 1,
  });

  let firstDayOfPesach = Temporal.PlainDate.from({
    calendar: "hebrew",
    year,
    monthCode: "M07",
    day: 15,
  });

  let yearSymbol = {
    353: "D",  // deficient
    354: "R",  // regular
    355: "C",  // complete

    383: "D",  // deficient, leap year
    384: "R",  // regular, leap year
    385: "C",  // complete, leap year
  };

  // Week starts on Sunday.
  let daySymbol = date => (date.dayOfWeek % 7) + 1;

  let {daysInYear} = startOfYear;
  assert.sameValue(daysInYear in yearSymbol, true);

  return `${daySymbol(startOfYear)}${yearSymbol[daysInYear]}${daySymbol(firstDayOfPesach)}`;
}

const validKeviahSymbols = new Set([
  "2D3", "2C5", "2D5", "2C7",
  "3R5", "3R7",
  "5R7", "5C1", "5D1", "5C3",
  "7D1", "7C3", "7D3", "7C5"
]);

for (let year = 3700; year <= 5800; ++year) {
  let sym = KeviahSymbol(year);
  assert.sameValue(validKeviahSymbols.has(sym), true, `${year} -> ${sym}`);
}
