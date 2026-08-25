// Copyright (c) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-global-environment-records-hasbinding-n
es6id: 8.1.1.4.1
description: >
  Symbol.unscopables is not referenced for the object environment of the global
  environment record
info: |
  1. Let envRec be the global Environment Record for which the method was invoked.
  2. Let DclRec be envRec.[[DeclarativeRecord]].
  3. If DclRec.HasBinding(N) is true, return true.
  4. Let ObjRec be envRec.[[ObjectRecord]].
  5. Return ? ObjRec.HasBinding(N).

  8.1.1.2.1 HasBinding

  1. Let envRec be the object Environment Record for which the method was
     invoked.
  2. Let bindings be the binding object for envRec.
  3. Let foundBinding be ? HasProperty(bindings, N).
  4. If foundBinding is false, return false.
  5. If the withEnvironment flag of envRec is false, return true.
  6. Let unscopables be ? Get(bindings, @@unscopables).
  7. If Type(unscopables) is Object, then
     a. Let blocked be ToBoolean(? Get(unscopables, N)).
     b. If blocked is true, return false.
  8. Return true.
features: [Symbol.unscopables]
---*/

var callCount = 0;
Object.defineProperty(this, Symbol.unscopables, {
  get: function() {
    callCount += 1;
  }
});

this.test262 = true;

test262;

assert.sameValue(
  callCount, 0, 'Did not reference @@unscopables property of global object'
);
