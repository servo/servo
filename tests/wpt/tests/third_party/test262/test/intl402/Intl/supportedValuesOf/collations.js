// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "collation" values are sorted, unique, and match the type production.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  3. Else if key is "collation", then
    a. Let list be ! AvailableCollations( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableCollations ( )
    The AvailableCollations abstract operation returns a List, ordered as if an
    Array of the same values had been sorted using %Array.prototype.sort% using
    undefined as comparefn, that contains unique collation types identifying the
    collations for which the implementation provides the functionality of
    Intl.Collator objects.
includes: [compareArray.js]
features: [Intl-enumeration, Intl.Locale, Array.prototype.includes]
---*/

const collations = Intl.supportedValuesOf("collation");

assert(Array.isArray(collations), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(collations), Array.prototype,
                 "The array prototype is Array.prototype");

const otherCollations = Intl.supportedValuesOf("collation");
assert.notSameValue(otherCollations, collations,
                    "Returns a new array object for each call.");

assert.compareArray(collations, otherCollations.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(collations).size, collations.length,
                 "The array doesn't contain duplicates.");

// https://unicode.org/reports/tr35/tr35.html#Unicode_locale_identifier
const typeRE = /^[a-z0-9]{3,8}(-[a-z0-9]{3,8})*$/;
for (let collation of collations) {
  assert(typeRE.test(collation), `${collation} matches the 'type' production`);
}

for (let collation of collations) {
  assert.sameValue(new Intl.Locale("und", {collation}).collation, collation,
                   `${collation} is canonicalised`);
}

assert(!collations.includes("standard"), "Mustn't include the 'standard' collation type.");
assert(!collations.includes("search"), "Mustn't include the 'search' collation type.");
