(function memory_fill0_wast_js() {

// memory_fill0.wast:2
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x87\x80\x80\x80\x00\x03\x00\x00\x00\x00\x00\x01\x07\x92\x80\x80\x80\x00\x02\x04\x66\x69\x6c\x6c\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8b\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0b\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x2d\x40\x02\x00\x0b", "memory_fill0.wast:2");

// memory_fill0.wast:2
let $1 = instance($$1);

// memory_fill0.wast:18
run(() => call($1, "fill", [1, 255, 3]), "memory_fill0.wast:18");

// memory_fill0.wast:19
assert_return(() => call($1, "load8_u", [0]), "memory_fill0.wast:19", 0);

// memory_fill0.wast:20
assert_return(() => call($1, "load8_u", [1]), "memory_fill0.wast:20", 255);

// memory_fill0.wast:21
assert_return(() => call($1, "load8_u", [2]), "memory_fill0.wast:21", 255);

// memory_fill0.wast:22
assert_return(() => call($1, "load8_u", [3]), "memory_fill0.wast:22", 255);

// memory_fill0.wast:23
assert_return(() => call($1, "load8_u", [4]), "memory_fill0.wast:23", 0);

// memory_fill0.wast:26
run(() => call($1, "fill", [0, 48_042, 2]), "memory_fill0.wast:26");

// memory_fill0.wast:27
assert_return(() => call($1, "load8_u", [0]), "memory_fill0.wast:27", 170);

// memory_fill0.wast:28
assert_return(() => call($1, "load8_u", [1]), "memory_fill0.wast:28", 170);

// memory_fill0.wast:31
run(() => call($1, "fill", [0, 0, 65_536]), "memory_fill0.wast:31");

// memory_fill0.wast:34
assert_trap(() => call($1, "fill", [65_280, 1, 257]), "memory_fill0.wast:34");

// memory_fill0.wast:36
assert_return(() => call($1, "load8_u", [65_280]), "memory_fill0.wast:36", 0);

// memory_fill0.wast:37
assert_return(() => call($1, "load8_u", [65_535]), "memory_fill0.wast:37", 0);

// memory_fill0.wast:40
run(() => call($1, "fill", [65_536, 0, 0]), "memory_fill0.wast:40");

// memory_fill0.wast:43
assert_trap(() => call($1, "fill", [65_537, 0, 0]), "memory_fill0.wast:43");
reinitializeRegistry();
})();
