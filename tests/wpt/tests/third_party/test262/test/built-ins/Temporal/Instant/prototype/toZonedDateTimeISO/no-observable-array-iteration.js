// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: >
  Calling GetPossibleEpochNanoseconds (from ToTemporalZonedDateTime > InterpretISODateTimeOffset)
  causes no observable array iteration.
features: [Temporal]
---*/

const arrayPrototypeSymbolIteratorOriginal = Array.prototype[Symbol.iterator];
Array.prototype[Symbol.iterator] = function arrayIterator() {
  throw new Test262Error("Array should not be iterated");
}

let inst = new Temporal.Instant(0n);
let zdt = inst.toZonedDateTimeISO("UTC");

Array.prototype[Symbol.iterator] = arrayPrototypeSymbolIteratorOriginal;
