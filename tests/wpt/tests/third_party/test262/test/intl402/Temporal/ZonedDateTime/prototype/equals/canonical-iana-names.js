// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Canonicalizes to evaluate time zone equality
features: [Temporal]
---*/

const neverEqual = new Temporal.ZonedDateTime(0n, 'Asia/Tokyo');
const zdt = new Temporal.ZonedDateTime(0n, 'America/Los_Angeles');
const ids = [
  ['America/Atka', 'America/Adak'],
  ['America/Knox_IN', 'America/Indiana/Knox'],
  ['Asia/Ashkhabad', 'Asia/Ashgabat'],
  ['Asia/Dacca', 'Asia/Dhaka'],
  ['Asia/Istanbul', 'Europe/Istanbul'],
  ['Asia/Macao', 'Asia/Macau'],
  ['Asia/Thimbu', 'Asia/Thimphu'],
  ['Asia/Ujung_Pandang', 'Asia/Makassar'],
  ['Asia/Ulan_Bator', 'Asia/Ulaanbaatar']
];

for (const [identifier, primaryIdentifier] of ids) {
  const z1 = zdt.withTimeZone(identifier);
  const z2 = zdt.withTimeZone(primaryIdentifier);

  // compare objects
  assert(z1.equals(z2), `${identifier} equals ${primaryIdentifier} object`);
  assert(z2.equals(z1), `${primaryIdentifier} equals ${identifier} object`);
  assert(!z1.equals(neverEqual), "not equal to unrelated time zone object");

  // compare IXDTF strings
  assert(z1.equals(z2.toString()), `${identifier} equals ${primaryIdentifier} IXDTF string`);
  assert(z2.equals(z1.toString()), `${primaryIdentifier} equals ${identifier} IXDTF string`);
  assert(!z1.equals(neverEqual.toString()), "not equal to unrelated IXDTF string");
}
