(function memory_init0_wast_js() {

// memory_init0.wast:2
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x89\x80\x80\x80\x00\x04\x00\x00\x00\x00\x00\x01\x00\x00\x07\x92\x80\x80\x80\x00\x02\x04\x69\x6e\x69\x74\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0c\x81\x80\x80\x80\x00\x01\x0a\x9f\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x08\x00\x02\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x2d\x40\x02\x00\x0b\x0b\x87\x80\x80\x80\x00\x01\x01\x04\xaa\xbb\xcc\xdd", "memory_init0.wast:2");

// memory_init0.wast:2
let $1 = instance($$1);

// memory_init0.wast:19
run(() => call($1, "init", [0, 1, 2]), "memory_init0.wast:19");

// memory_init0.wast:20
assert_return(() => call($1, "load8_u", [0]), "memory_init0.wast:20", 187);

// memory_init0.wast:21
assert_return(() => call($1, "load8_u", [1]), "memory_init0.wast:21", 204);

// memory_init0.wast:22
assert_return(() => call($1, "load8_u", [2]), "memory_init0.wast:22", 0);

// memory_init0.wast:25
run(() => call($1, "init", [65_532, 0, 4]), "memory_init0.wast:25");

// memory_init0.wast:28
assert_trap(() => call($1, "init", [65_534, 0, 3]), "memory_init0.wast:28");

// memory_init0.wast:30
assert_return(() => call($1, "load8_u", [65_534]), "memory_init0.wast:30", 204);

// memory_init0.wast:31
assert_return(() => call($1, "load8_u", [65_535]), "memory_init0.wast:31", 221);

// memory_init0.wast:34
run(() => call($1, "init", [65_536, 0, 0]), "memory_init0.wast:34");

// memory_init0.wast:35
run(() => call($1, "init", [0, 4, 0]), "memory_init0.wast:35");

// memory_init0.wast:38
assert_trap(() => call($1, "init", [65_537, 0, 0]), "memory_init0.wast:38");

// memory_init0.wast:40
assert_trap(() => call($1, "init", [0, 5, 0]), "memory_init0.wast:40");
reinitializeRegistry();
})();
