// This file was procedurally generated from the following sources:
// - src/class-elements/static-field-anonymous-function-length.case
// - src/class-elements/default/cls-expr.template
/*---
description: Anonymous function in field initilizer have length properly set (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-static-fields-private, class-static-fields-public, class]
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
      ...

---*/


var C = class {
  static #field = (a, b) => undefined;
  static field = function() {};

  static accessPrivateField() {
    return this.#field;
  }

}

assert.sameValue(C.accessPrivateField().length, 2);
assert.sameValue(C.field.length, 0);
