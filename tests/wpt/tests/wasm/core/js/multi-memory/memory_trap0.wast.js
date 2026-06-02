(function memory_trap0_wast_js() {

// memory_trap0.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8f\x80\x80\x80\x00\x03\x60\x00\x01\x7f\x60\x02\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x85\x80\x80\x80\x00\x04\x00\x01\x02\x02\x05\x87\x80\x80\x80\x00\x03\x00\x00\x00\x00\x00\x01\x07\x9e\x80\x80\x80\x00\x03\x05\x73\x74\x6f\x72\x65\x00\x01\x04\x6c\x6f\x61\x64\x00\x02\x0b\x6d\x65\x6d\x6f\x72\x79\x2e\x67\x72\x6f\x77\x00\x03\x0a\xbc\x80\x80\x80\x00\x04\x89\x80\x80\x80\x00\x00\x3f\x02\x41\x80\x80\x04\x6c\x0b\x8d\x80\x80\x80\x00\x00\x10\x00\x20\x00\x6a\x20\x01\x36\x42\x02\x00\x0b\x8b\x80\x80\x80\x00\x00\x10\x00\x20\x00\x6a\x28\x42\x02\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x02\x0b", "memory_trap0.wast:1");

// memory_trap0.wast:1
let $1 = instance($$1);

// memory_trap0.wast:23
assert_return(() => call($1, "store", [-4, 42]), "memory_trap0.wast:23");

// memory_trap0.wast:24
assert_return(() => call($1, "load", [-4]), "memory_trap0.wast:24", 42);

// memory_trap0.wast:25
assert_trap(() => call($1, "store", [-3, 305_419_896]), "memory_trap0.wast:25");

// memory_trap0.wast:26
assert_trap(() => call($1, "load", [-3]), "memory_trap0.wast:26");

// memory_trap0.wast:27
assert_trap(() => call($1, "store", [-2, 13]), "memory_trap0.wast:27");

// memory_trap0.wast:28
assert_trap(() => call($1, "load", [-2]), "memory_trap0.wast:28");

// memory_trap0.wast:29
assert_trap(() => call($1, "store", [-1, 13]), "memory_trap0.wast:29");

// memory_trap0.wast:30
assert_trap(() => call($1, "load", [-1]), "memory_trap0.wast:30");

// memory_trap0.wast:31
assert_trap(() => call($1, "store", [0, 13]), "memory_trap0.wast:31");

// memory_trap0.wast:32
assert_trap(() => call($1, "load", [0]), "memory_trap0.wast:32");

// memory_trap0.wast:33
assert_trap(() => call($1, "store", [-2_147_483_648, 13]), "memory_trap0.wast:33");

// memory_trap0.wast:34
assert_trap(() => call($1, "load", [-2_147_483_648]), "memory_trap0.wast:34");

// memory_trap0.wast:35
assert_return(() => call($1, "memory.grow", [65_537]), "memory_trap0.wast:35", -1);
reinitializeRegistry();
})();
