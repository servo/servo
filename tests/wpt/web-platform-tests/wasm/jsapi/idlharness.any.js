// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=../resources/load_wasm.js

'use strict';

// https://webassembly.github.io/spec/js-api/

promise_test(async () => {
  const srcs = ['wasm-js-api'];
  const [wasm] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(wasm, {
    // Note the prose requirements in the specification.
    except: ['CompileError', 'LinkError', 'RuntimeError']
  });

  // Ignored errors are surfaced in idlharness.js's test_object below.
  try {
    self.memory = new WebAssembly.Memory({initial: 1024});
  } catch (e) { }

  try {
    self.mod = await createWasmModule();
    self.instance = new WebAssembly.Instance(self.mod);
  } catch (e) { }

  idl_array.add_objects({
    Memory: ['memory'],
    Module: ['mod'],
    Instance: ['instance'],
  });
  idl_array.test();
}, 'wasm-js-api interfaces.');

