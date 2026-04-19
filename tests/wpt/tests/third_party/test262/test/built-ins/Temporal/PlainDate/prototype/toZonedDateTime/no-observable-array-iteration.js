// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Calling GetPossibleEpochNanoseconds (from ToTemporalZonedDateTime > InterpretISODateTimeOffset)
  causes no observable array iteration.
features: [Temporal]
---*/

const arrayPrototypeSymbolIteratorOriginal = Array.prototype[Symbol.iterator];
Array.prototype[Symbol.iterator] = function arrayIterator() {
  throw new Test262Error("Array should not be iterated");
}

let pd = new Temporal.PlainDate(2000, 1, 1);
let zdt = pd.toZonedDateTime("UTC");

Array.prototype[Symbol.iterator] = arrayPrototypeSymbolIteratorOriginal;
