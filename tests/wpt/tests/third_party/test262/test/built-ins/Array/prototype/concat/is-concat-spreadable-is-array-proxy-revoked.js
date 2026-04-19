// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-isconcatspreadable
description: Revoked proxy value produces a TypeError when supplied to IsArray
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
  3. If spreadable is not undefined, return ToBoolean(spreadable).
  4. Return ? IsArray(O).

  7.2.2 IsArray

  [...]
  3. If argument is a Proxy exotic object, then
     a. If the value of the [[ProxyHandler]] internal slot of argument is null,
        throw a TypeError exception.
     b. Let target be the value of the [[ProxyTarget]] internal slot of
        argument.
     c. Return ? IsArray(target).
features: [Proxy, Symbol.isConcatSpreadable]
---*/

var target = [];
var handle = Proxy.revocable(target, {
  get: function(_, key) {
    // Defer proxy revocation until after property access to ensure that the
    // expected TypeError originates from the IsArray operation.
    if (key === Symbol.isConcatSpreadable) {
      handle.revoke();
    }
    return target[key];
  }
});

assert.throws(TypeError, function() {
  [].concat(handle.proxy);
}, '[].concat(handle.proxy) throws a TypeError exception');
