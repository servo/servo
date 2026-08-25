// This file was procedurally generated from the following sources:
// - src/class-elements/computed-name-toprimitive-err.case
// - src/class-elements/class-evaluation-error/cls-decl.template
/*---
description: Custom error evaluating a computed property name (field definitions in a class declaration)
esid: sec-runtime-semantics-classdefinitionevaluation
features: [class-fields-public, computed-property-names, Symbol.toPrimitive, class]
flags: [generated]
info: |
    Runtime Semantics: ClassDefinitionEvaluation

    ...
    27. For each ClassElement e in order from elements
      a. If IsStatic of me is false, then
        i. Let fields be the result of performing ClassElementEvaluation for e with arguments proto and false.
      b. Else,
        i. Let fields be the result of performing ClassElementEvaluation for e with arguments F and false.
      c. If fields is an abrupt completion, then
        i. Set the running execution context's LexicalEnvironment to lex.
        ii. Set the running execution context's PrivateNameEnvironment to outerPrivateEnvironment.
        iii. Return Completion(status).
    ...

    Runtime Semantics: ClassElementEvaluation

    ClassElement: FieldDefinition;
      Return ClassFieldDefinitionEvaluation of FieldDefinition with parameter false and object.

    Runtime Semantics: ClassFieldDefinitionEvaluation
      With parameters isStatic and homeObject.

    1. Let fieldName be the result of evaluating ClassElementName.
    2. ReturnIfAbrupt(fieldName).
    ...

    Runtime Semantics: Evaluation
      ComputedPropertyName: [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
    3. Return ? ToPropertyKey(propName).

---*/
var obj = {
  [Symbol.toPrimitive]: function() {
    throw new Test262Error();
  }
};



function evaluate() {
  class C {
    [obj]
  }
}

assert.throws(Test262Error, evaluate);
