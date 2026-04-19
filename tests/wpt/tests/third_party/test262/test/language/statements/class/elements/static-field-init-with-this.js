// This file was procedurally generated from the following sources:
// - src/class-elements/static-field-init-with-this.case
// - src/class-elements/default/cls-decl.template
/*---
description: Static fields initializer has `this` biding (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-static-fields-public, class]
flags: [generated]
info: |
    Updated Productions

    ClassElement :
      ...
      static FieldDefinition ;

    FieldDefinition :
      ClassElementName Initializer_opt

    ClassDefinitionEvaluation:
      ...

      27. Let staticFields be a new empty List.
      28. For each ClassElement e in order from elements,
        a. If IsStatic of e is false, then
        ...
        b. Else,
          i. Let field be the result of performing PropertyDefinitionEvaluation for m ClassElementEvaluation for e with arguments F and false.
        c. If field is an abrupt completion, then
          ...
        d. If field is not empty,
          i. If IsStatic of e is false, append field to instanceFields.
          ii. Otherwise, append field to staticFields.

      34. For each item fieldRecord in order from staticFields,
        a. Perform ? DefineField(F, field).
      ...

    DefineField(receiver, fieldRecord)
      1. Assert: Type(receiver) is Object.
      2. Assert: fieldRecord is a Record as created by ClassFieldDefinitionEvaluation.
      3. Let name be fieldRecord.[[Name]].
      4. Let initializer be fieldRecord.[[Initializer]].
      5. If initializer is not empty, then
        a. Let initValue be ? Call(initializer, receiver).
      6. Else, let initValue be undefined.
      7. If fieldRecord.[[IsAnonymousFunctionDefinition]] is true, then
        a. Let hasNameProperty be ? HasOwnProperty(initValue, "name").
        b. If hasNameProperty is false, perform SetFunctionName(initValue, fieldName).
      8. If fieldName is a Private Name,
        a. Perform ? PrivateFieldAdd(fieldName, receiver, initValue).
      9. Else,
        a. Assert: IsPropertyKey(fieldName) is true.
        b. Perform ? CreateDataPropertyOrThrow(receiver, fieldName, initValue).
      10. Return.

---*/


class C {
  static f = 'test';
  static g = this.f + '262';
  static h = eval('this.g') + 'test';
}

assert.sameValue(C.f, 'test');
assert.sameValue(C.g, 'test262');
assert.sameValue(C.h, 'test262test');
