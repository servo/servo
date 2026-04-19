// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createresolvingfunctions
description: >
  reject is anonymous built-in function with length of 1.
info: |
  CreateResolvingFunctions ( promise )

  ...
  7. Let reject be ! CreateBuiltinFunction(stepsReject, « [[Promise]], [[AlreadyResolved]] »).
features: [Reflect.construct, arrow-function]
includes: [isConstructor.js]
flags: [async]
---*/

Promise.resolve(1).then(function() {
  return Promise.resolve();
}).then($DONE, $DONE);

var then = Promise.prototype.then;
Promise.prototype.then = function(resolve, reject) {
  assert.sameValue(isConstructor(reject), false, 'isConstructor(reject) must return false');
  assert.throws(TypeError, () => {
    new reject();
  });

  assert.sameValue(reject.length, 1, 'The value of reject.length is 1');
  assert.sameValue(reject.name, '', 'The value of reject.name is ""');

  return then.call(this, resolve, reject);
};
