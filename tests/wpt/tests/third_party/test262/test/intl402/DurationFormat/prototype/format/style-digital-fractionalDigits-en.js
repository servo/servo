// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondurationformatpattern
description: >
  Test to ensure that correct number of fractional digits is displayed if fractionalDigits is explicitly specified.

info: |
  4. If durationFormat.[[FractionalDigits]] is undefined, then
    a. Perform ! CreateDataPropertyOrThrow(nfOpts, "maximumFractionDigits", 9).
    b. Perform ! CreateDataPropertyOrThrow(nfOpts, "minimumFractionDigits", 0).
  5. Else,
    a. Perform ! CreateDataPropertyOrThrow(nfOpts, "maximumFractionDigits", durationFormat.[[FractionalDigits]]).
    b. Perform ! CreateDataPropertyOrThrow(nfOpts, "minimumFractionDigits", durationFormat.[[FractionalDigits]]).
locale: [en]
features: [Intl.DurationFormat]
---*/

const duration = {
  hours: 1,
  minutes: 22,
  seconds: 33,
  milliseconds: 111,
  microseconds: 222,
  nanoseconds: 333,
};


const style = "digital";
const df = new Intl.DurationFormat("en", {style, fractionalDigits: 0});
const dfMilli = new Intl.DurationFormat("en", {style, fractionalDigits: 3});
const dfFourDigits = new Intl.DurationFormat("en", {style, fractionalDigits: 4});
const dfMicro = new Intl.DurationFormat("en", {style, fractionalDigits: 6});
const dfEightDigits = new Intl.DurationFormat("en", {style, fractionalDigits: 8});
const dfNano = new Intl.DurationFormat("en", {style, fractionalDigits: 9});

assert.sameValue(df.format(duration), "1:22:33", `format output without sub-second digits using ${style} style option`);

assert.sameValue(dfMilli.format(duration), "1:22:33.111", `format output with sub-second digits and fractionalDigits: 3 using ${style} style option`);

assert.sameValue(dfFourDigits.format(duration), "1:22:33.1112", `format output with sub-second digits and fractionalDigits: 4 using ${style} style option`);

assert.sameValue(dfMicro.format(duration), "1:22:33.111222", `format output with sub-second digits and fractionalDigits: 6 using ${style} style option`);

assert.sameValue(dfEightDigits.format(duration), "1:22:33.11122233", `format output with sub-second digits and fractionalDigits: 8 using ${style} style option`);

assert.sameValue(dfNano.format(duration), "1:22:33.111222333", `format output with sub-second digits and fractionalDigits: 9 using ${style} style option`);
