// This file was procedurally generated from the following sources:
// - src/class-elements/fields-anonymous-function-length.case
// - src/class-elements/default/cls-expr.template
/*---
description: Anonymous functions in field initializer have length properly set (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-private, class-fields-public, class]
flags: [generated]
info: |
    Updated Productions

    FieldDefinition :
      ClassElementName Initializer_opt

    InitializeInstanceFields ( O, constructor )
      1. Assert: Type ( O ) is Object.
      2. Assert: constructor is an ECMAScript function object.
      3. Let fields be the value of constructor.[[Fields]].
      4. For each item fieldRecord in order from fields,
        a. Perform ? DefineField(O, fieldRecord).
      5. Return.

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
  field = function() {};
  #field = (a, b, c, d) => undefined;

  accessPrivateField() {
    return this.#field;
  }
}

let c = new C();
assert.sameValue(c.accessPrivateField().length, 4);
assert.sameValue(c.field.length, 0);
