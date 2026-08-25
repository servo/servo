// This file was procedurally generated from the following sources:
// - src/dynamic-import/ns-get-nested-namespace-dflt-direct.case
// - src/dynamic-import/namespace/promise.template
/*---
description: Direct Default exports are included in an imported module namespace object when a namespace object is created. (value from promise then)
esid: sec-finishdynamicimport
features: [export-star-as-namespace-from-module, dynamic-import]
flags: [generated, async]
info: |
    Runtime Semantics: FinishDynamicImport ( referencingScriptOrModule, specifier, promiseCapability, completion )

        1. If completion is an abrupt completion, ...
        2. Otherwise,
            ...
            d. Let namespace be GetModuleNamespace(moduleRecord).
            e. If namespace is an abrupt completion, perform ! Call(promiseCapability.[[Reject]], undefined, « namespace.[[Value]] »).
            f. Otherwise, perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace.[[Value]] »).

    Runtime Semantics: GetModuleNamespace ( module )

        ...
        3. Let namespace be module.[[Namespace]].
        4. If namespace is undefined, then
            a. Let exportedNames be ? module.GetExportedNames(« »).
            b. Let unambiguousNames be a new empty List.
            c. For each name that is an element of exportedNames, do
                i. Let resolution be ? module.ResolveExport(name, « »).
                ii. If resolution is a ResolvedBinding Record, append name to unambiguousNames.
            d. Set namespace to ModuleNamespaceCreate(module, unambiguousNames).
        5. Return namespace.

    ModuleNamespaceCreate ( module, exports )

        ...
        4. Let M be a newly created object.
        5. Set M's essential internal methods to the definitions specified in 9.4.6.
        7. Let sortedExports be a new List containing the same values as the list exports where the
        values are ordered as if an Array of the same values had been sorted using Array.prototype.sort
        using undefined as comparefn.
        8. Set M.[[Exports]] to sortedExports.
        9. Create own properties of M corresponding to the definitions in 26.3.
        10. Set module.[[Namespace]] to M.
        11. Return M.

    26.3 Module Namespace Objects

        A Module Namespace Object is a module namespace exotic object that provides runtime
        property-based access to a module's exported bindings. There is no constructor function for
        Module Namespace Objects. Instead, such an object is created for each module that is imported
        by an ImportDeclaration that includes a NameSpaceImport.

        In addition to the properties specified in 9.4.6 each Module Namespace Object has the
        following own property:

    26.3.1 @@toStringTag

        The initial value of the @@toStringTag property is the String value "Module".

        This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.

    Module Namespace Exotic Objects

        A module namespace object is an exotic object that exposes the bindings exported from an
        ECMAScript Module (See 15.2.3). There is a one-to-one correspondence between the String-keyed
        own properties of a module namespace exotic object and the binding names exported by the
        Module. The exported bindings include any bindings that are indirectly exported using export *
        export items. Each String-valued own property key is the StringValue of the corresponding
        exported binding name. These are the only String-keyed properties of a module namespace exotic
        object. Each such property has the attributes { [[Writable]]: true, [[Enumerable]]: true,
        [[Configurable]]: false }. Module namespace objects are not extensible.


    [...]
    6. Let binding be ! m.ResolveExport(P, « »).
    7. Assert: binding is a ResolvedBinding Record.
    8. Let targetModule be binding.[[Module]].
    9. Assert: targetModule is not undefined.
    10. If binding.[[BindingName]] is "*namespace*", then
    11. Return ? GetModuleNamespace(targetModule).

    Runtime Semantics: GetModuleNamespace
    [...]
      3. If namespace is undefined, then
         a. Let exportedNames be ? module.GetExportedNames(« »).
         b. Let unambiguousNames be a new empty List.
         c. For each name that is an element of exportedNames,
            i. Let resolution be ? module.ResolveExport(name, « », « »).
            ii. If resolution is null, throw a SyntaxError exception.
            iii. If resolution is not "ambiguous", append name to
                 unambiguousNames.
         d. Let namespace be ModuleNamespaceCreate(module, unambiguousNames).
    [...]

---*/

import('./get-nested-namespace-dflt-skip-prod_FIXTURE.js').then(ns => {

    var desc = Object.getOwnPropertyDescriptor(ns, 'productionNS2');

    assert.sameValue(desc.enumerable, true, 'ns.productionNS2: is enumerable');
    assert.sameValue(desc.writable, true, 'ns.productionNS2: is writable');
    assert.sameValue(desc.configurable, false, 'ns.productionNS2: is non-configurable');

    var keys = Object.getOwnPropertyNames(ns.productionNS2);

    assert.sameValue(keys.length, 2);
    assert.sameValue(keys[0], 'default');
    assert.sameValue(keys[1], 'productionOther');

    desc = Object.getOwnPropertyDescriptor(ns.productionNS2, 'productionOther');

    assert.sameValue(desc.value, null, 'ns.productionNS2.productionOther: value is null');
    assert.sameValue(desc.enumerable, true, 'ns.productionNS2.productionOther: is enumerable');
    assert.sameValue(desc.writable, true, 'ns.productionNS2.productionOther: is writable');
    assert.sameValue(desc.configurable, false, 'ns.productionNS2.productionOther: is non-configurable');

    desc = Object.getOwnPropertyDescriptor(ns.productionNS2, 'default');

    assert.sameValue(desc.value, 42, 'ns.productionNS2.default value is 42');
    assert.sameValue(desc.enumerable, true, 'ns.productionNS2.default is enumerable');
    assert.sameValue(desc.writable, true, 'ns.productionNS2.default is writable');
    assert.sameValue(desc.configurable, false, 'ns.productionNS2.default is non-configurable');

}).then($DONE, $DONE).catch($DONE);
