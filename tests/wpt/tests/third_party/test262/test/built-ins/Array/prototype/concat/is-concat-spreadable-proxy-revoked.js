// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-isconcatspreadable
description: >
  Revoked proxy value produces a TypeError during access of
  `Symbol.isConcatSpreadable` property
info: |
  [...]
  5. Repeat, while items is not empty
     a. Remove the first element from items and let E be the value of the
        element.
     b. Let spreadable be ? IsConcatSpreadable(E).
     c. If spreadable is true, then
        [...]
     e. Else E is added as a single item rather than spread,
        [...]

  ES6 22.1.3.1.1: Runtime Semantics: IsConcatSpreadable ( O )

  1. If Type(O) is not Object, return false.
  2. Let spreadable be ? Get(O, @@isConcatSpreadable).

  7.3.1 Get

  [...]
  3. Return ? O.[[Get]](P, O).

  9.5.8 [[Get]]

  [...]
  2. Let handler be O.[[ProxyHandler]].
  3. If handler is null, throw a TypeError exception.
features: [Proxy]
---*/

var handle = Proxy.revocable([], {});

handle.revoke();

assert.throws(TypeError, function() {
  [].concat(handle.proxy);
}, '[].concat(handle.proxy) throws a TypeError exception');
