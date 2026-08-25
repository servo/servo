(function memory_copy1_wast_js() {

// memory_copy1.wast:2
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x03\x7f\x7f\x7f\x00\x60\x01\x7f\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x8d\x80\x80\x80\x00\x04\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x07\x92\x80\x80\x80\x00\x02\x04\x63\x6f\x70\x79\x00\x00\x07\x6c\x6f\x61\x64\x38\x5f\x75\x00\x01\x0a\x9e\x80\x80\x80\x00\x02\x8c\x80\x80\x80\x00\x00\x20\x00\x20\x01\x20\x02\xfc\x0a\x00\x03\x0b\x87\x80\x80\x80\x00\x00\x20\x00\x2d\x00\x00\x0b\x0b\xa8\x80\x80\x80\x00\x04\x00\x41\x00\x0b\x04\xff\x11\x44\xee\x02\x01\x41\x00\x0b\x04\xee\x22\x55\xff\x02\x02\x41\x00\x0b\x04\xdd\x33\x66\x00\x02\x03\x41\x00\x0b\x04\xaa\xbb\xcc\xdd", "memory_copy1.wast:2");

// memory_copy1.wast:2
let $1 = instance($$1);

// memory_copy1.wast:19
run(() => call($1, "copy", [10, 0, 4]), "memory_copy1.wast:19");

// memory_copy1.wast:21
assert_return(() => call($1, "load8_u", [9]), "memory_copy1.wast:21", 0);

// memory_copy1.wast:22
assert_return(() => call($1, "load8_u", [10]), "memory_copy1.wast:22", 170);

// memory_copy1.wast:23
assert_return(() => call($1, "load8_u", [11]), "memory_copy1.wast:23", 187);

// memory_copy1.wast:24
assert_return(() => call($1, "load8_u", [12]), "memory_copy1.wast:24", 204);

// memory_copy1.wast:25
assert_return(() => call($1, "load8_u", [13]), "memory_copy1.wast:25", 221);

// memory_copy1.wast:26
assert_return(() => call($1, "load8_u", [14]), "memory_copy1.wast:26", 0);

// memory_copy1.wast:29
run(() => call($1, "copy", [65_280, 0, 256]), "memory_copy1.wast:29");

// memory_copy1.wast:30
run(() => call($1, "copy", [65_024, 65_280, 256]), "memory_copy1.wast:30");

// memory_copy1.wast:33
run(() => call($1, "copy", [65_536, 0, 0]), "memory_copy1.wast:33");

// memory_copy1.wast:34
run(() => call($1, "copy", [0, 65_536, 0]), "memory_copy1.wast:34");

// memory_copy1.wast:37
assert_trap(() => call($1, "copy", [65_537, 0, 0]), "memory_copy1.wast:37");

// memory_copy1.wast:39
assert_trap(() => call($1, "copy", [0, 65_537, 0]), "memory_copy1.wast:39");
reinitializeRegistry();
})();
