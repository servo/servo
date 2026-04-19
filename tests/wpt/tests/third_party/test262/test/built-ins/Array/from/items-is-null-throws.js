// Copyright 2015 Leonardo Balter. All rights reserved.
// This code is governed by the license found in the LICENSE file.
/*---
esid: sec-array.from
description: Throws a TypeError if items argument is null
info: |
  22.1.2.1 Array.from ( items [ , mapfn [ , thisArg ] ] )

  ...
  4. Let usingIterator be GetMethod(items, @@iterator).
  5. ReturnIfAbrupt(usingIterator).
  ...
---*/

assert.throws(TypeError, function() {
  Array.from(null);
}, 'Array.from(null) throws a TypeError exception');
