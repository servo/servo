// Copyright (C) 2024 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  %WrapForValidIteratorPrototype%.return() should return an iterator result object that value is undefined when base object does not have return method.
info: |
  %WrapForValidIteratorPrototype%.return ( )
    ...
    5. Let returnMethod be ? GetMethod(iterator, "return").
    6. If returnMethod is undefined, then
      a. Return CreateIterResultObject(undefined, true).

features: [iterator-helpers]
---*/

const iter = {};
const wrapper = Iterator.from(iter);

const result = wrapper.return();
assert(result.hasOwnProperty("value"));
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
