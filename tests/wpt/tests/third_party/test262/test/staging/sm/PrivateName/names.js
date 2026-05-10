// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var C = class {
  static #field = () => 'Test262';
  static field = () => 'Test262';
  #instance = () => 'Test262';
  instance = () => 'Test262';

  static accessPrivateField() {
    return this.#field;
  }

  accessPrivateInstanceField() {
    return this.#instance;
  }

  static accessField() {
    return this.field;
  }

  accessInstanceField() {
    return this.instance;
  }
}
assert.sameValue(C.accessPrivateField().name, '#field')
assert.sameValue(C.accessField().name, 'field');
var c = new C;
assert.sameValue(c.accessPrivateInstanceField().name, '#instance');
assert.sameValue(c.accessInstanceField().name, 'instance');

