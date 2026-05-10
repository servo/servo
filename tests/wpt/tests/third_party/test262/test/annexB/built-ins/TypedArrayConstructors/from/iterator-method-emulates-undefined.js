// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  [[IsHTMLDDA]] object as @@iterator method gets called.
info: |
  %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  [...]
  5. Let usingIterator be ? GetMethod(items, @@iterator).
  6. If usingIterator is not undefined, then
    a. Let values be ? IterableToList(source, usingIterator).

  IterableToList ( items, method )

  1. Let iteratorRecord be ? GetIterator(items, sync, method).

  GetIterator ( obj [ , hint [ , method ] ] )

  [...]
  4. Let iterator be ? Call(method, obj).
  5. If Type(iterator) is not Object, throw a TypeError exception.
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray, IsHTMLDDA]
---*/

var items = {};
items[Symbol.iterator] = $262.IsHTMLDDA;

testWithTypedArrayConstructors(function(TypedArray) {
  assert.throws(TypeError, function() {
    TypedArray.from(items);
  });
});
