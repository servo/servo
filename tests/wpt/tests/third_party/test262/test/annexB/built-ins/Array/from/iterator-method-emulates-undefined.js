// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: >
  [[IsHTMLDDA]] object as @@iterator method gets called.
info: |
  Array.from ( items [ , mapfn [ , thisArg ] ] )

  [...]
  4. Let usingIterator be ? GetMethod(items, @@iterator).
  5. If usingIterator is not undefined, then
    [...]
    c. Let iteratorRecord be ? GetIterator(items, sync, usingIterator).

  GetIterator ( obj [ , hint [ , method ] ] )

  [...]
  4. Let iterator be ? Call(method, obj).
  5. If Type(iterator) is not Object, throw a TypeError exception.
features: [Symbol.iterator, IsHTMLDDA]
---*/

var items = {};
items[Symbol.iterator] = $262.IsHTMLDDA;

assert.throws(TypeError, function() {
  Array.from(items);
});
