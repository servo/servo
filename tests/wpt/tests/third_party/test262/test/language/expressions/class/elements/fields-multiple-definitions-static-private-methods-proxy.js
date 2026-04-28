// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Static private methods not accessible via default Proxy handler
esid: prod-FieldDefinition
features: [class, class-static-methods-private]
info: |
    ClassElement :
      ...
      static FieldDefinition ;

    FieldDefinition :
      ClassElementName Initializer_opt

    ClassElementName :
      PrivateName

    PrivateName :
      # IdentifierName

---*/


var C = class {
  static #x(value) {
    return 1;
  }
  static x() {
    return this.#x();
  }
}

var P = new Proxy(C, {});

assert.sameValue(C.x(), 1);
assert.throws(TypeError, function() {
  P.x();
});
