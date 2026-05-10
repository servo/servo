// This file was procedurally generated from the following sources:
// - src/identifier-names/extends-escaped.case
// - src/identifier-names/default/arrow-fn-assignment-identifier.template
/*---
description: extends is a valid identifier name, using escape (IdentifierReference in ObjectAssignmentPattern (Arrow Function) cannot be a ReservedWord)
esid: prod-AssignmentPattern
features: [arrow-function, destructuring-assignment]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    AssignmentPattern:
      ObjectAssignmentPattern

    ObjectAssignmentPattern:
      { AssignmentPropertyList }

    AssignmentPropertyList:
      AssignmentProperty
      AssignmentPropertyList , AssignmentProperty

    AssignmentProperty:
      IdentifierReference Initializer_opt
      PropertyName : AssignmentElement

    IdentifierReference:
      Identifier
      [~Yield]yield
      [~Await]await

    Identifier:
      IdentifierName but not ReservedWord

---*/


$DONOTEVALUATE();

var x = ({ \u0065xtends }) => {};
