// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private setter is visible on initializer with direct eval
esid: sec-privatefieldset
info: |
  PrivateFieldSet (P, O, value )
    1. Assert: P is a Private Name.
    2. If O is not an object, throw a TypeError exception.
    3. If P.[[Kind]] is "field",
      a. Let entry be PrivateFieldFind(P, O).
      b. If entry is empty, throw a TypeError exception.
      c. Set entry.[[PrivateFieldValue]] to value.
      d. Return.
    4. If P.[[Kind]] is "method", throw a TypeError exception.
    5. Else,
      a. Assert: P.[[Kind]] is "accessor".
      b. If O.[[PrivateFieldBrands]] does not contain P.[[Brand]], throw a TypeError exception.
      c. If P does not have a [[Set]] field, throw a TypeError exception.
      d. Let setter be P.[[Set]].
      e. Perform ? Call(setter, O, value).
      f. Return.

  ClassElementName : PrivateIdentifier
    1. Let privateIdentifier be StringValue of PrivateIdentifier.
    2. Let privateName be NewPrivateName(privateIdentifier).
    3. Let scope be the running execution context's PrivateEnvironment.
    4. Let scopeEnvRec be scope's EnvironmentRecord.
    5. Perform ! scopeEnvRec.InitializeBinding(privateIdentifier, privateName).
    6. Return privateName.

  MakePrivateReference ( baseValue, privateIdentifier )
    1. Let env be the running execution context's PrivateEnvironment.
    2. Let privateNameBinding be ? ResolveBinding(privateIdentifier, env).
    3. Let privateName be GetValue(privateNameBinding).
    4. Assert: privateName is a Private Name.
    5. Return a value of type Reference whose base value is baseValue, whose referenced name is privateName, whose strict reference flag is true.
features: [class-fields-public, class-methods-private, class]
---*/

class C {
  set #m(v) { this._v = v; };
  v = (eval("this.#m = 53"), this._v);
}

let c = new C();
assert.sameValue(c.v, 53);
