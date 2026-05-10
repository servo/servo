// Copyright 2019 Google Inc., 2023 Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.resolvedOptions
description: >
  Intl.DateTimeFormat.prototype.resolvedOptions properly
  reflect hourCycle settings.
info: |
  11.3.7 Intl.DateTimeFormat.prototype.resolvedOptions()

  11.1.2 CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )
   23. Let dataLocaleData be localeData.[[<dataLocale>]].
   24. If hour12 is true, then
       a. Let hc be dataLocaleData.[[hourCycle12]].
   25. Else if hour12 is false, then
       a. Let hc be dataLocaleData.[[hourCycle24]].
   26. Else,
        a. Assert: hour12 is undefined.
        b. Let hc be r.[[hc]].
        c. If hc is null, set hc to dataLocaleData.[[hourCycle]].
  27. Set dateTimeFormat.[[HourCycle]] to hc.

locale: [en, fr, it, ja, zh, ko, ar, hi]
---*/

let locales = ["en", "fr", "it", "ja", "zh", "ko", "ar", "hi"];

for (let locale of locales) {
  let hcDefault = new Intl.DateTimeFormat(locale, {hour: "numeric"}).resolvedOptions().hourCycle;
  assert(
    hcDefault === "h11" || hcDefault === "h12" || hcDefault === "h23" || hcDefault === "h24",
    "hcDefault is one of [h11, h12, h23, h24]"
  );

  let hour12 = new Intl.DateTimeFormat(locale, {hour: "numeric", hour12: true}).resolvedOptions().hourCycle;
  assert(hour12 === "h11" || hour12 === "h12", "hour12 is one of [h11, h12]");

  let hour24 = new Intl.DateTimeFormat(locale, {hour: "numeric", hour12: false}).resolvedOptions().hourCycle;
  assert(hour24 === "h23" || hour24 === "h24", "hour24 is one of [h23, h24]");

  if (hcDefault === "h11" || hcDefault === "h12") {
    assert.sameValue(hour12, hcDefault, "hour12 matches hcDefault");
  } else {
    assert.sameValue(hour24, hcDefault, "hour24 matches hcDefault");
  }

  // 24-hour clock uses the "h23" format in all locales.
  assert.sameValue(hour24, "h23");

  // 12-hour clock uses the "h12" format in all locales except "ja".
  assert.sameValue(hour12, locale === "ja" ? "h11" : "h12");
}
