// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator
description: >
  Attributes in Unicode extension subtags should be ignored.
info: |
  Intl.Collator ( [ locales [ , options ] ] )

  1. If _r_.[[co]] is *null*, let _collation_ be *"default"*. Otherwise, let
     _collation_ be _r_.[[co]].
  1. Set _collator_.[[Collation]] to _collation_.

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
