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
    ...
    15. Set the running execution context's LexicalEnvironment to classScope.
    16. Set the running execution context's PrivateEnvironment to classPrivateEnvironment.
    ...
    27. For each ClassElement e in order from elements
      a. If IsStatic of e is false, then
        i. Let field be the result of ClassElementEvaluation for e with arguments proto and false.
    ...

  FieldDefinition : ClassElementName Initializer
    1. Let name be the result of evaluating ClassElementName.
    ...

  ClassElementName : PrivateIdentifier
    1. Let privateIdentifier be StringValue of PrivateIdentifier.
    2. Let privateName be NewPrivateName(privateIdentifier).
    3. Let scope be the running execution context's PrivateEnvironment.
    4. Let scopeEnvRec be scope's EnvironmentRecord.
    5. Perform ! scopeEnvRec.InitializeBinding(privateIdentifier, privateName).
    6. Return privateName.

  MemberExpression : MemberExpression . PrivateIdentifier
    ...
    5. Return MakePrivateReference(bv, fieldNameString).

  MakePrivateReference ( baseValue, privateIdentifier )
    ...
    2. Let privateNameBinding be ? ResolveBinding(privateIdentifier, env).
    3. Let privateName be GetValue(privateNameBinding).
    ...

  GetValue (V)
    ...
    5. If IsPropertyReference(V), then
      a. If HasPrimitiveBase(V), then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ToObject(base).
      b. If IsPrivateReference(V), then
        i. Return ? PrivateFieldGet(GetReferencedName(V), base).
    6. Else,
      a. Assert: base is an Environment Record.
      b. Return ? base.GetBindingValue(GetReferencedName(V), IsStrictReference(V)).

  PrivateFieldGet (P, O)
    1. Assert: P is a Private Name.
    2. Assert: Type(O) is Object.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.
    5. Return entry.[[PrivateFieldValue]].

features: [class-fields-private, class-fields-public, class]
---*/

const self = this;
assert.throws(TypeError, function() {
  class C {
    #f = 'foo';
    [self.#f] = 'Test262';
  }
}, 'access to a not defined private field in object should throw a TypeError');

