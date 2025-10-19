(function table_grow64_wast_js() {

// table_grow64.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x95\x80\x80\x80\x00\x04\x60\x01\x7e\x01\x6f\x60\x02\x7e\x6f\x00\x60\x02\x7e\x6f\x01\x7e\x60\x00\x01\x7e\x03\x85\x80\x80\x80\x00\x04\x00\x01\x02\x03\x04\x84\x80\x80\x80\x00\x01\x6f\x04\x00\x07\xab\x80\x80\x80\x00\x04\x07\x67\x65\x74\x2d\x74\x36\x34\x00\x00\x07\x73\x65\x74\x2d\x74\x36\x34\x00\x01\x08\x67\x72\x6f\x77\x2d\x74\x36\x34\x00\x02\x08\x73\x69\x7a\x65\x2d\x74\x36\x34\x00\x03\x0a\xb1\x80\x80\x80\x00\x04\x86\x80\x80\x80\x00\x00\x20\x00\x25\x00\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x20\x01\x26\x00\x0b\x89\x80\x80\x80\x00\x00\x20\x01\x20\x00\xfc\x0f\x00\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b", "table_grow64.wast:1");

// table_grow64.wast:1
let $1 = instance($$1);

// table_grow64.wast:12
assert_return(() => call($1, "size-t64", []), "table_grow64.wast:12", 0n);

// table_grow64.wast:13
assert_trap(() => call($1, "set-t64", [0n, hostref(2)]), "table_grow64.wast:13");

// table_grow64.wast:14
assert_trap(() => call($1, "get-t64", [0n]), "table_grow64.wast:14");

// table_grow64.wast:16
assert_return(() => call($1, "grow-t64", [1n, null]), "table_grow64.wast:16", 0n);

// table_grow64.wast:17
assert_return(() => call($1, "size-t64", []), "table_grow64.wast:17", 1n);

// table_grow64.wast:18
assert_return(() => call($1, "get-t64", [0n]), "table_grow64.wast:18", null);

// table_grow64.wast:19
assert_return(() => call($1, "set-t64", [0n, hostref(2)]), "table_grow64.wast:19");

// table_grow64.wast:20
assert_return(() => call($1, "get-t64", [0n]), "table_grow64.wast:20", hostref(2));

// table_grow64.wast:21
assert_trap(() => call($1, "set-t64", [1n, hostref(2)]), "table_grow64.wast:21");

// table_grow64.wast:22
assert_trap(() => call($1, "get-t64", [1n]), "table_grow64.wast:22");

// table_grow64.wast:24
assert_return(() => call($1, "grow-t64", [4n, hostref(3)]), "table_grow64.wast:24", 1n);

// table_grow64.wast:25
assert_return(() => call($1, "size-t64", []), "table_grow64.wast:25", 5n);

// table_grow64.wast:26
assert_return(() => call($1, "get-t64", [0n]), "table_grow64.wast:26", hostref(2));

// table_grow64.wast:27
assert_return(() => call($1, "set-t64", [0n, hostref(2)]), "table_grow64.wast:27");

// table_grow64.wast:28
assert_return(() => call($1, "get-t64", [0n]), "table_grow64.wast:28", hostref(2));

// table_grow64.wast:29
assert_return(() => call($1, "get-t64", [1n]), "table_grow64.wast:29", hostref(3));

// table_grow64.wast:30
assert_return(() => call($1, "get-t64", [4n]), "table_grow64.wast:30", hostref(3));

// table_grow64.wast:31
assert_return(() => call($1, "set-t64", [4n, hostref(4)]), "table_grow64.wast:31");

// table_grow64.wast:32
assert_return(() => call($1, "get-t64", [4n]), "table_grow64.wast:32", hostref(4));

// table_grow64.wast:33
assert_trap(() => call($1, "set-t64", [5n, hostref(2)]), "table_grow64.wast:33");

// table_grow64.wast:34
assert_trap(() => call($1, "get-t64", [5n]), "table_grow64.wast:34");
reinitializeRegistry();
})();
