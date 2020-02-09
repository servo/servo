const instanceTestFactory = [
  [
    "Empty module without imports argument",
    function() {
      return {
        buffer: emptyModuleBinary,
        args: [],
        exports: {},
        verify: () => {},
      };
    }
  ],

  [
    "Empty module with undefined imports argument",
    function() {
      return {
        buffer: emptyModuleBinary,
        args: [undefined],
        exports: {},
        verify: () => {},
      };
    }
  ],

  [
    "Empty module with empty imports argument",
    function() {
      return {
        buffer: emptyModuleBinary,
        args: [{}],
        exports: {},
        verify: () => {},
      };
    }
  ],

  [
    "getter order for imports object",
    function() {
      const builder = new WasmModuleBuilder();
      builder.addImportedGlobal("module", "global1", kWasmI32);
      builder.addImportedGlobal("module2", "global3", kWasmI32);
      builder.addImportedMemory("module", "memory", 0, 128);
      builder.addImportedGlobal("module", "global2", kWasmI32);
      const buffer = builder.toBuffer();
      const order = [];

      const imports = {
        get module() {
          order.push("module getter");
          return {
            get global1() {
              order.push("global1 getter");
              return 0;
            },
            get global2() {
              order.push("global2 getter");
              return 0;
            },
            get memory() {
              order.push("memory getter");
              return new WebAssembly.Memory({ "initial": 64, maximum: 128 });
            },
          }
        },
        get module2() {
          order.push("module2 getter");
          return {
            get global3() {
              order.push("global3 getter");
              return 0;
            },
          }
        },
      };

      const expected = [
        "module getter",
        "global1 getter",
        "module2 getter",
        "global3 getter",
        "module getter",
        "memory getter",
        "module getter",
        "global2 getter",
      ];
      return {
        buffer,
        args: [imports],
        exports: {},
        verify: () => assert_array_equals(order, expected),
      };
    }
  ],

  [
    "imports",
    function() {
      const builder = new WasmModuleBuilder();

      builder.addImport("module", "fn", kSig_v_v);
      builder.addImportedGlobal("module", "global", kWasmI32);
      builder.addImportedMemory("module", "memory", 0, 128);
      builder.addImportedTable("module", "table", 0, 128);

      const buffer = builder.toBuffer();
      const imports = {
        "module": {
          "fn": function() {},
          "global": 0,
          "memory": new WebAssembly.Memory({ "initial": 64, maximum: 128 }),
          "table": new WebAssembly.Table({ "element": "anyfunc", "initial": 64, maximum: 128 }),
        },
        get "module2"() {
          assert_unreached("Should not get modules that are not imported");
        },
      };

      return {
        buffer,
        args: [imports],
        exports: {},
        verify: () => {},
      };
    }
  ],

  [
    "No imports",
    function() {
      const builder = new WasmModuleBuilder();

      builder
        .addFunction("fn", kSig_v_d)
        .addBody([])
        .exportFunc();
      builder
        .addFunction("fn2", kSig_v_v)
        .addBody([])
        .exportFunc();

      builder.setTableBounds(1);
      builder.addExportOfKind("table", kExternalTable, 0);

      builder.addGlobal(kWasmI32, true)
        .exportAs("global")
        .init = 7;
      builder.addGlobal(kWasmF64, true)
        .exportAs("global2")
        .init = 1.2;

      builder.addMemory(4, 8, true);

      const buffer = builder.toBuffer();

      const exports = {
        "fn": { "kind": "function", "name": "0", "length": 1 },
        "fn2": { "kind": "function", "name": "1", "length": 0 },
        "table": { "kind": "table", "length": 1 },
        "global": { "kind": "global", "value": 7 },
        "global2": { "kind": "global", "value": 1.2 },
        "memory": { "kind": "memory", "size": 4 },
      };

      return {
        buffer,
        args: [],
        exports,
        verify: () => {},
      };
    }
  ],

  [
    "exports and imports",
    function() {
      const value = 102;

      const builder = new WasmModuleBuilder();

      const index = builder.addImportedGlobal("module", "global", kWasmI32);
      builder
        .addFunction("fn", kSig_i_v)
        .addBody([
            kExprGlobalGet,
            index,
            kExprReturn,
        ])
        .exportFunc();

      const buffer = builder.toBuffer();

      const imports = {
        "module": {
          "global": value,
        },
      };

      const exports = {
        "fn": { "kind": "function", "name": "0", "length": 0 },
      };

      return {
        buffer,
        args: [imports],
        exports,
        verify: instance => assert_equals(instance.exports.fn(), value)
      };
    }
  ],

  [
    "stray argument",
    function() {
      return {
        buffer: emptyModuleBinary,
        args: [{}, {}],
        exports: {},
        verify: () => {}
      };
    }
  ],
];
