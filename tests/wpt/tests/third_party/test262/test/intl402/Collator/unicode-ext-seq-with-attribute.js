// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializecollator
description: >
  Attributes in Unicode extension subtags should be ignored.
info: |
  10.1.1 InitializeCollator ( collator, locales, options )
    ...
    15. For each element key of relevantExtensionKeys in List order, do
      a. If key is "co", then
          i. Let value be r.[[co]].
         ii. If value is null, let value be "default".
        iii. Set collator.[[Collation]] to value.
    ...

  10.3.5 Intl.Collator.prototype.resolvedOptions ()
    The function returns a new object whose properties and attributes are set as if constructed
    by an object literal assigning to each of the following properties the value of the
    corresponding internal slot of this Collator object (see 10.4): ...
---*/

var colExpected = new Intl.Collator("de-u-attrval-co-phonebk");
var colActual = new Intl.Collator("de-u-co-phonebk");

var resolvedExpected = colExpected.resolvedOptions();
var resolvedActual = colActual.resolvedOptions();

assert.sameValue(resolvedActual.locale, resolvedExpected.locale);
assert.sameValue(resolvedActual.collation, resolvedExpected.collation);
