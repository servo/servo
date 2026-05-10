// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: finally on resolved Promise calls the SpeciesConstructor
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
---*/

class FooPromise extends Promise {
  static get[Symbol.species]() {
    return Promise;
  }
}

var p = Promise.resolve().finally(() => FooPromise.resolve());

assert.sameValue(p instanceof Promise, true);
assert.sameValue(p instanceof FooPromise, false);
