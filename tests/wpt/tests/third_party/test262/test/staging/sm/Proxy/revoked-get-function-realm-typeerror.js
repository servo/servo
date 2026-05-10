// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var constructors = [
    // 19.1 Object Objects
    {constructor: Object},

    // 19.2 Function Objects
    {constructor: Function},

    // 19.3 Boolean Objects
    {constructor: Boolean},

    // 19.5 Error Objects
    {constructor: Error},

    // 19.5.5 Native Error Types Used in This Standard
    {constructor: EvalError},
    {constructor: RangeError},
    {constructor: ReferenceError},
    {constructor: SyntaxError},
    {constructor: TypeError},
    {constructor: URIError},

    // 20.1 Number Objects
    {constructor: Number},

    // 20.3 Date Objects
    {constructor: Date},

    // 21.1 String Objects
    {constructor: String},

    // 21.2 RegExp (Regular Expression) Objects
    {constructor: RegExp},

    // 22.1 Array Objects
    {constructor: Array},

    // 22.2 TypedArray Objects
    {constructor: Int8Array},

    // 23.1 Map Objects
    {constructor: Map},

    // 23.2 Set Objects
    {constructor: Set},

    // 23.3 WeakMap Objects
    {constructor: WeakMap},

    // 23.4 WeakSet Objects
    {constructor: WeakSet},

    // 24.1 ArrayBuffer Objects
    {constructor: ArrayBuffer},

    // 24.2 SharedArrayBuffer Objects
    ...(typeof SharedArrayBuffer === "function" ? [{constructor: SharedArrayBuffer}] : []),

    // 24.3 DataView Objects
    {constructor: DataView, args: [new ArrayBuffer(0)]},

    // 25.2 GeneratorFunction Objects
    {constructor: function*(){}.constructor},

    // 25.3 AsyncGeneratorFunction Objects
    {constructor: async function*(){}.constructor},

    // 25.6 Promise Objects
    {constructor: Promise, args: [function(){}]},

    // 25.7 AsyncFunction Objects
    {constructor: async function(){}.constructor},

    // 9.2 ECMAScript Function Objects
    {constructor: function(){}},

    // Intl can be disabled at compile-time.
    ...(typeof Intl !== "undefined" ? [
        // 10 Collator Objects
        {constructor: Intl.Collator},

        // 11 NumberFormat Objects
        {constructor: Intl.NumberFormat},

        // 12 DateTimeFormat Objects
        {constructor: Intl.DateTimeFormat},

        // 13 PluralRules Objects
        {constructor: Intl.PluralRules},

        // Intl.RelativeTimeFormat proposal
        {constructor: Intl.RelativeTimeFormat},

        // Intl.Locale is not yet enabled by default.
        ...(Intl.Locale ? [Intl.Locale] : []),
    ] : []),
];

for (let {constructor, args = []} of constructors) {
    let revoked = 0;
    let {proxy, revoke} = Proxy.revocable(function(){}, {
        get(t, pk, r) {
            if (pk === "prototype") {
                revoked++;
                revoke();
                return undefined;
            }
            return Reflect.get(t, pk, r);
        }
    });

    assert.throws(TypeError, () => {
        Reflect.construct(constructor, args, proxy);
    });

    assert.sameValue(revoked, 1);
}

