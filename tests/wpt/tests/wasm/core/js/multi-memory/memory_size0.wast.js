(function memory_size0_wast_js() {

// memory_size0.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x01\x7f\x00\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x8b\x80\x80\x80\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x07\x8f\x80\x80\x80\x00\x02\x04\x73\x69\x7a\x65\x00\x00\x04\x67\x72\x6f\x77\x00\x01\x0a\x96\x80\x80\x80\x00\x02\x84\x80\x80\x80\x00\x00\x3f\x04\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x40\x04\x1a\x0b", "memory_size0.wast:1");

// memory_size0.wast:1
let $1 = instance($$1);

// memory_size0.wast:12
assert_return(() => call($1, "size", []), "memory_size0.wast:12", 0);

// memory_size0.wast:13
assert_return(() => call($1, "grow", [1]), "memory_size0.wast:13");

// memory_size0.wast:14
assert_return(() => call($1, "size", []), "memory_size0.wast:14", 1);

// memory_size0.wast:15
assert_return(() => call($1, "grow", [4]), "memory_size0.wast:15");

// memory_size0.wast:16
assert_return(() => call($1, "size", []), "memory_size0.wast:16", 5);

// memory_size0.wast:17
assert_return(() => call($1, "grow", [0]), "memory_size0.wast:17");

// memory_size0.wast:18
assert_return(() => call($1, "size", []), "memory_size0.wast:18", 5);
reinitializeRegistry();
})();
