// Copyright 2015 Leonardo Balter. All rights reserved.
// This code is governed by the license found in the LICENSE file.
/*---
esid: sec-array.from
description: Return empty array if items argument is an ArrayBuffer
info: |
  22.1.2.1 Array.from ( items [ , mapfn [ , thisArg ] ] )

  ...
  4. Let usingIterator be GetMethod(items, @@iterator).
  5. ReturnIfAbrupt(usingIterator).
  ...
---*/

var arrayBuffer = new ArrayBuffer(7);

var result = Array.from(arrayBuffer);

assert.sameValue(result.length, 0, 'The value of result.length is expected to be 0');
