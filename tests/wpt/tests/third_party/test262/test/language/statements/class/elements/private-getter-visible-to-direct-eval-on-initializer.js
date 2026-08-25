// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private getter is visible on initializer with direct eval
esid: sec-privatefieldget
info: |
  PrivateFieldGet (P, O)
    1. Assert: P is a Private Name.
    2. If O is not an object, throw a TypeError exception.
    3. If P.[[Kind]] is "field",
      a. Let entry be PrivateFieldFind(P, O).
      b. If entry is empty, throw a TypeError exception.
      c. Return entry.[[PrivateFieldValue]].
    4. Perform ? PrivateBrandCheck(O, P).
    5. If P.[[Kind]] is "method",
      a. Return P.[[Value]].
    6. Else,
      a. Assert: P.[[Kind]] is "accessor".
      b. If P does not have a [[Get]] field, throw a TypeError exception.
      c. Let getter be P.[[Get]].
      d. Return ? Call(getter, O).

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
features: [class-methods-private, class-fields-public, class]
---*/

class C {
  get #m() { return "Test262"; };
  v = eval("this.#m");
}

let c = new C();
assert.sameValue(c.v, "Test262");
