// This file was procedurally generated from the following sources:
// - src/function-forms/forbidden-ext-direct-access-prop-arguments.case
// - src/function-forms/forbidden-extensions/bullet-one/async-gen-meth.template
/*---
description: Forbidden extension, f.arguments (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [arrow-function, async-iteration, generators]
flags: [generated, noStrict, async]
info: |
    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }


    ECMAScript function objects defined using syntactic constructors in strict mode code must
    not be created with own properties named "caller" or "arguments". Such own properties also
    must not be created for function objects defined using an ArrowFunction, MethodDefinition,
    GeneratorDeclaration, GeneratorExpression, AsyncGeneratorDeclaration, AsyncGeneratorExpression,
    ClassDeclaration, ClassExpression, AsyncFunctionDeclaration, AsyncFunctionExpression, or
    AsyncArrowFunction regardless of whether the definition is contained in strict mode code.
    Built-in functions, strict functions created using the Function constructor, generator functions
    created using the Generator constructor, async functions created using the AsyncFunction
    constructor, and functions created using the bind method also must not be created with such own
    properties.

---*/

var callCount = 0;
var obj = {
  async *method() {
    assert.sameValue(this.method.hasOwnProperty("arguments"), false);
    callCount++;
  }
};

obj.method().next()
  .then(() => {
    assert.sameValue(callCount, 1, 'function body evaluated');
  }, $DONE).then($DONE, $DONE);
