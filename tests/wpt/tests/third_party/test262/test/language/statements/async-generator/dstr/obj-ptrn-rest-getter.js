// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [object-rest, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]

---*/
var count = 0;


var callCount = 0;
async function* f({...x}) {
  assert.sameValue(count, 1);

  verifyProperty(x, "v", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  callCount = callCount + 1;
};
f({ get v() { count++; return 2; } }).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
