// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.6.1
description: Instances has the own property lastIndex
info: |
  21.2.6.1 lastIndex

  The value of the lastIndex property specifies the String index at which to
  start the next match. It is coerced to an integer when used (see 21.2.5.2.2).
  This property shall have the attributes { [[Writable]]: true, [[Enumerable]]:
  false, [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

class RE extends RegExp {}

var re = new RE('39?');

re.exec('TC39');

verifyProperty(re, 'lastIndex', {
  value: 0,
  writable: true,
  enumerable: false,
  configurable: false,
});
