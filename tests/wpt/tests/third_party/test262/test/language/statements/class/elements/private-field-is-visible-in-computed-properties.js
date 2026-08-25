// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: PrivateName of a class is visible in its ComputetProperty scope
esid: prod-ClassTail
info: |
  ClassTail : ClassHeritage { ClassBody }
    1. Let lex be the LexicalEnvironment of the running execution context.
    2. Let classScope be NewDeclarativeEnvironment(lex).
    3. Let classScopeEnvRec be classScope's EnvironmentRecord.
    ...
    8. If ClassBodyopt is present, then
        a. For each element dn of the PrivateBoundIdentifiers of ClassBodyopt,
          i. Perform classPrivateEnvRec.CreateImmutableBinding(dn, true).
          ii. Let privateName be NewPrivateName(dn).
          iii. Perform ! classPrivateEnvRec.InitializeBinding(dn, dn).
    ...
    15. Set the running execution context's LexicalEnvironment to classScope.
    16. Set the running execution context's PrivateEnvironment to classPrivateEnvironment.
    ...
    27. For each ClassElement e in order from elements
      a. If IsStatic of e is false, then
        i. Let field be the result of ClassElementEvaluation for e with arguments proto and false.
    ...

  GetValue ( V )
    ...
    5. If IsPropertyReference(V), then
      ...
      b. If IsPrivateReference(V), then
        i. Return ? PrivateFieldGet(GetReferencedName(V), base).

  PrivateFieldGet ( P, O )
    ...
    4. If entry is empty, throw a TypeError exception.
    ...

features: [class-fields-private, class-fields-public, class]
---*/

const self = this;
assert.throws(TypeError, function() {
  class C {
    [self.#f] = 'Test262';
    #f = 'foo';
  }
}, 'access to a not defined private field in object should throw a TypeError');

