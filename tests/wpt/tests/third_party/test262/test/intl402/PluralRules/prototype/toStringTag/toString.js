// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules.prototype-tostringtag
description: >
  Object.prototype.toString utilizes Intl.PluralRules.prototype[@@toStringTag].
info: |
  Object.prototype.toString ( )

  [...]
  14. Else, let builtinTag be "Object".
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".

  Intl.PluralRules.prototype [ @@toStringTag ]

  The initial value of the @@toStringTag property is the String value "Intl.PluralRules".
features: [Symbol.toStringTag]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.PluralRules.prototype), "[object Intl.PluralRules]");
assert.sameValue(Object.prototype.toString.call(new Intl.PluralRules()), "[object Intl.PluralRules]");
