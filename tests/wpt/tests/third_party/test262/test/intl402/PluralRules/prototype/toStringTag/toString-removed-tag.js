// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules.prototype-tostringtag
description: >
  Object.prototype.toString doesn't special-case neither Intl.PluralRules instances nor its prototype.
info: |
  Object.prototype.toString ( )

  [...]
  14. Else, let builtinTag be "Object".
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".
features: [Symbol.toStringTag]
---*/

delete Intl.PluralRules.prototype[Symbol.toStringTag];

assert.sameValue(Object.prototype.toString.call(Intl.PluralRules.prototype), "[object Object]");
assert.sameValue(Object.prototype.toString.call(new Intl.PluralRules()), "[object Object]");
