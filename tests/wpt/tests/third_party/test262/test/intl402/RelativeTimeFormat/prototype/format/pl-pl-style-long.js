// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the behavior of Intl.RelativeTimeFormat.prototype.format() in Polish.
features: [Intl.RelativeTimeFormat]
locale: [pl-PL]
---*/

function regular(s) {
  return {
    "many": s,
    "few": s + "y",
    "one": s + "ę",
  }
}

// https://www.unicode.org/cldr/charts/33/summary/pl.html#1419
const units = {
  "second": regular("sekund"),
  "minute": regular("minut"),
  "hour": regular("godzin"),
  "day": {
    "many": "dni",
    "few": "dni",
    "one": "dzień",
  },
  "week": {
    "many": "tygodni",
    "few": "tygodnie",
    "one": "tydzień",
  },
  "month": {
    1000: "miesięcy",
    "many": "miesięcy",
    "few": "miesiące",
    "one": "miesiąc",
  },
  "quarter": {
    "many": "kwartałów",
    "few": "kwartały",
    "one": "kwartał",
  },
  "year": {
    "many": "lat",
    "few": "lata",
    "one": "rok",
  },
};

const rtf = new Intl.RelativeTimeFormat("pl-PL", {
  "style": "long",
});

assert.sameValue(typeof rtf.format, "function", "format should be supported");

for (const [unitArgument, expected] of Object.entries(units)) {
  assert.sameValue(rtf.format(1000, unitArgument), `za 1000 ${expected.many}`);
  assert.sameValue(rtf.format(10, unitArgument), `za 10 ${expected.many}`);
  assert.sameValue(rtf.format(2, unitArgument), `za 2 ${expected.few}`);
  assert.sameValue(rtf.format(1, unitArgument), `za 1 ${expected.one}`);
  assert.sameValue(rtf.format(0, unitArgument), `za 0 ${expected.many}`);
  assert.sameValue(rtf.format(-0, unitArgument), `0 ${expected.many} temu`);
  assert.sameValue(rtf.format(-1, unitArgument), `1 ${expected.one} temu`);
  assert.sameValue(rtf.format(-2, unitArgument), `2 ${expected.few} temu`);
  assert.sameValue(rtf.format(-10, unitArgument), `10 ${expected.many} temu`);
  assert.sameValue(rtf.format(-1000, unitArgument), `1000 ${expected.many} temu`);
}
