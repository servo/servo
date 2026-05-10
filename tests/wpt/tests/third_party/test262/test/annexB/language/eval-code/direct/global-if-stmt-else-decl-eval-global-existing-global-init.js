// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-existing-global-init.case
// - src/annex-b-fns/eval-global/direct-if-stmt-else-decl.template
/*---
description: Variable binding is left in place by legacy function hoisting (IfStatement with a declaration in the second statement position in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    i. If varEnvRec is a global Environment Record, then
       i. Perform ? varEnvRec.CreateGlobalVarBinding(F, true).
    [...]

---*/
Object.defineProperty(fnGlobalObject(), 'f', {
  value: 'x',
  enumerable: true,
  writable: true,
  configurable: false
});

eval(
  'var global = fnGlobalObject();\
  assert.sameValue(f, "x", "binding is not reinitialized");\
  \
  verifyProperty(global, "f", {\
    enumerable: true,\
    writable: true,\
    configurable: false\
  }, { restore: true });if (false) ; else function f() {  }'
);

assert.sameValue(typeof f, "function");
verifyProperty(global, "f", {
  enumerable: true,
  writable: true,
  configurable: false
});
