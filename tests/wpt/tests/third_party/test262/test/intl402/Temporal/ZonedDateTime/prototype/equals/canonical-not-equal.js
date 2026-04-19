// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Canonical time zone identifiers are never equal to each other
features: [Temporal, Intl-enumeration]
---*/

// supportedValuesOf only returns canonical IDs
const ids = Intl.supportedValuesOf("timeZone");

const forEachDistinctPair = (array, func) => {
  for (let i = 0; i < array.length; i++) {
    for (let j = i + 1; j < array.length; j++) {
      func(array[i], array[j]);
    }
  }
};

forEachDistinctPair(ids, (id1, id2) => {
  const instance = new Temporal.ZonedDateTime(0n, id1);
  assert(!instance.equals(instance.withTimeZone(id2)), `${id1} does not equal ${id2}`);
})

