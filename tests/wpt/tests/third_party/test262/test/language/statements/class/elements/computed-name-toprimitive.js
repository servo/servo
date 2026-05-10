// This file was procedurally generated from the following sources:
// - src/class-elements/computed-name-toprimitive.case
// - src/class-elements/default/cls-decl.template
/*---
description: ToPrimitive evaluation in the ComputedPropertyName (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, Symbol.toPrimitive, class]
flags: [generated]
includes: [propertyHelper.js]
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
var err = function() { throw new Test262Error(); };
var obj1 = {
  [Symbol.toPrimitive]: function() { return "d"; },
  toString: err,
  valueOf: err
};

var obj2 = {
  toString: function() { return "e"; },
  valueOf: err
};

var obj3 = {
  toString: undefined,
  valueOf: function() { return "f"; }
};



class C {
  [obj1] = 42;
  [obj2] = 43;
  [obj3] = 44;
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "d"),
  "d doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "d"),
  "d doesn't appear as an own property on C constructor"
);

verifyProperty(c, "d", {
  value: 42,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "e"),
  "e doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "e"),
  "e doesn't appear as an own property on C constructor"
);

verifyProperty(c, "e", {
  value: 43,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "f"),
  "f doesn't appear as an own property on C prototype"
);
assert(!
  Object.prototype.hasOwnProperty.call(C, "f"),
  "f doesn't appear as an own property on C constructor"
);

verifyProperty(c, "f", {
  value: 44,
  enumerable: true,
  writable: true,
  configurable: true
});
