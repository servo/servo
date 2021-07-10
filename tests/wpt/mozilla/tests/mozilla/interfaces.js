function test_interfaces(interfaceNamesInGlobalScope) {
  test(function() {
    // This is a list of interfaces that are exposed to every webpage by SpiderMonkey.
    // IMPORTANT: Do not change this list without review from a JavaScript Engine peer!
    var ecmaGlobals = [
      "AggregateError",
      "Array",
      "ArrayBuffer",
      "BigInt",
      "BigInt64Array",
      "BigUint64Array",
      "Boolean",
      "BroadcastChannel",
      "ByteLengthQueuingStrategy",
      "CountQueuingStrategy",
      "Crypto",
      "DataView",
      "Date",
      "Error",
      "EvalError",
      "Float32Array",
      "Float64Array",
      "Function",
      "Infinity",
      "Int16Array",
      "Int32Array",
      "Int8Array",
      "InternalError",
      "Intl",
      "JSON",
      "Map",
      "Math",
      "MessageChannel",
      "MessagePort",
      "NaN",
      "Number",
      "Object",
      "Promise",
      "Proxy",
      "RangeError",
      "ReadableStream",
      "ReferenceError",
      "Reflect",
      "RegExp",
      "Set",
      "String",
      "Symbol",
      "SyntaxError",
      "TextMetrics",
      "TypeError",
      "URIError",
      "Uint16Array",
      "Uint32Array",
      "Uint8Array",
      "Uint8ClampedArray",
      "WeakMap",
      "WeakSet",
      "WebAssembly",
    ];

    var sources = [
      ecmaGlobals,
      interfaceNamesInGlobalScope,
      ["AssertionError", "EventWatcher", "OptionalFeatureUnsupportedError"],
    ];

    var interfaceMap = {};
    for (var source of sources) {
      for (var entry of source) {
        interfaceMap[entry] = true;
      }
    }

    for (var name of Object.getOwnPropertyNames(self)) {
      if (!/^[A-Z]/.test(name) && name != 'console') {
        continue;
      }
      assert_true(name in interfaceMap,
                  "If this is failing: DANGER, are you sure you want to expose the new " +
                  "interface " + name + " to all webpages as a property on the global? " +
                  "Do not make a change to this file without review from jdm or Ms2ger " +
                  "for that specific change!");
      if (name in interfaceMap) {
        delete interfaceMap[name];
      }
    }
    for (var name of Object.keys(interfaceMap)) {
      assert_true(name in self, name + " should be defined on the global scope");
    }
    assert_equals(Object.keys(interfaceMap).length, 0,
                  "The following interface(s) are not enumerated: " +
                  Object.keys(interfaceMap).join(", "));
  });
}
