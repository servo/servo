// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-skip-early-err-for-in.case
// - src/annex-b-fns/func/if-decl-else-stmt.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (for-of statement) (IfStatement with a declaration in the first statement position in function scope)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    ii. If replacing the FunctionDeclaration f with a VariableStatement that
        has F as a BindingIdentifier would not produce any Early Errors for
        func and F is not an element of BoundNames of argumentsList, then
    [...]
---*/

(function() {
  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created prior to evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created prior to evaluation'
  );

  for (let f in { key: 0 }) {

  if (true) function f() {  } else ;

  }

  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created following evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created following evaluation'
  );
}());
