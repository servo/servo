// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.collations
description: >
    Checks that the return value of Intl.Locale.prototype.collations is an Array
    that does not contain invalid values.
info: |
  CollationsOfLocale ( loc )
  ...
  4. Let list be a List of 1 or more unique collation identifiers, which must
  be lower case String values conforming to the type sequence from UTS 35
  Unicode Locale Identifier, section 3.2, sorted in descending preference of
  those in common use for string comparison in locale. The values "standard"
  and "search" must be excluded from list.
features: [Intl.Locale, Intl.Locale-info, Array.prototype.includes]
---*/

const output = new Intl.Locale('en').getCollations();
assert(output.length > 0, 'array has at least one element');
output.forEach(c => {
  if(['standard', 'search'].includes(c))
    throw new Test262Error();
});
