// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise.race must throw TypeError per
    CreatePromiseCapabilityRecord step 8 when
    promiseCapabliity.[[Resolve]] is not callable
es6id: S25.4.4.3_A3.1_T2
author: Sam Mikes
description: Promise.race throws TypeError, even on empty array, when 'this' does not conform to Promise constructor
---*/

function BadPromiseConstructor(f) {
  f(undefined, undefined);
}

assert.throws(TypeError, function() {
  Promise.race.call(BadPromiseConstructor, []);
});
