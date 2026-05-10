// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 22.1.4.1
description: >
  Instances has the own property length
info: |
  22.1.4.1 length

  The length property of an Array instance is a data property whose value is
  always numerically greater than the name of every configurable own property
  whose name is an array index.

  The length property initially has the attributes { [[Writable]]: true,
  [[Enumerable]]: false, [[Configurable]]: false }.
---*/

class Ar extends Array {}

var arr = new Ar('foo', 'bar');

assert.sameValue(arr[0], 'foo');
assert.sameValue(arr[1], 'bar');

var arrDesc = Object.getOwnPropertyDescriptor(arr, 'length');

assert.sameValue(arrDesc.writable, true);
assert.sameValue(arrDesc.enumerable, false);
assert.sameValue(arrDesc.configurable, false);

assert.sameValue(arr[1], 'bar');

arr.length = 1;

assert.sameValue(arr[0], 'foo');
assert.sameValue(arr[1], undefined);
