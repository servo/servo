(function memory_grow64_wast_js() {

// memory_grow64.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x91\x80\x80\x80\x00\x04\x60\x00\x01\x7f\x60\x00\x00\x60\x01\x7e\x01\x7e\x60\x00\x01\x7e\x03\x87\x80\x80\x80\x00\x06\x00\x01\x00\x01\x02\x03\x05\x83\x80\x80\x80\x00\x01\x04\x00\x07\xd7\x80\x80\x80\x00\x06\x0c\x6c\x6f\x61\x64\x5f\x61\x74\x5f\x7a\x65\x72\x6f\x00\x00\x0d\x73\x74\x6f\x72\x65\x5f\x61\x74\x5f\x7a\x65\x72\x6f\x00\x01\x11\x6c\x6f\x61\x64\x5f\x61\x74\x5f\x70\x61\x67\x65\x5f\x73\x69\x7a\x65\x00\x02\x12\x73\x74\x6f\x72\x65\x5f\x61\x74\x5f\x70\x61\x67\x65\x5f\x73\x69\x7a\x65\x00\x03\x04\x67\x72\x6f\x77\x00\x04\x04\x73\x69\x7a\x65\x00\x05\x0a\xcd\x80\x80\x80\x00\x06\x87\x80\x80\x80\x00\x00\x42\x00\x28\x02\x00\x0b\x89\x80\x80\x80\x00\x00\x42\x00\x41\x02\x36\x02\x00\x0b\x89\x80\x80\x80\x00\x00\x42\x80\x80\x04\x28\x02\x00\x0b\x8b\x80\x80\x80\x00\x00\x42\x80\x80\x04\x41\x03\x36\x02\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b\x84\x80\x80\x80\x00\x00\x3f\x00\x0b", "memory_grow64.wast:1");

// memory_grow64.wast:1
let $1 = instance($$1);

// memory_grow64.wast:14
assert_return(() => call($1, "size", []), "memory_grow64.wast:14", 0n);

// memory_grow64.wast:15
assert_trap(() => call($1, "store_at_zero", []), "memory_grow64.wast:15");

// memory_grow64.wast:16
assert_trap(() => call($1, "load_at_zero", []), "memory_grow64.wast:16");

// memory_grow64.wast:17
assert_trap(() => call($1, "store_at_page_size", []), "memory_grow64.wast:17");

// memory_grow64.wast:18
assert_trap(() => call($1, "load_at_page_size", []), "memory_grow64.wast:18");

// memory_grow64.wast:19
assert_return(() => call($1, "grow", [1n]), "memory_grow64.wast:19", 0n);

// memory_grow64.wast:20
assert_return(() => call($1, "size", []), "memory_grow64.wast:20", 1n);

// memory_grow64.wast:21
assert_return(() => call($1, "load_at_zero", []), "memory_grow64.wast:21", 0);

// memory_grow64.wast:22
assert_return(() => call($1, "store_at_zero", []), "memory_grow64.wast:22");

// memory_grow64.wast:23
assert_return(() => call($1, "load_at_zero", []), "memory_grow64.wast:23", 2);

// memory_grow64.wast:24
assert_trap(() => call($1, "store_at_page_size", []), "memory_grow64.wast:24");

// memory_grow64.wast:25
assert_trap(() => call($1, "load_at_page_size", []), "memory_grow64.wast:25");

// memory_grow64.wast:26
assert_return(() => call($1, "grow", [4n]), "memory_grow64.wast:26", 1n);

// memory_grow64.wast:27
assert_return(() => call($1, "size", []), "memory_grow64.wast:27", 5n);

// memory_grow64.wast:28
assert_return(() => call($1, "load_at_zero", []), "memory_grow64.wast:28", 2);

// memory_grow64.wast:29
assert_return(() => call($1, "store_at_zero", []), "memory_grow64.wast:29");

// memory_grow64.wast:30
assert_return(() => call($1, "load_at_zero", []), "memory_grow64.wast:30", 2);

// memory_grow64.wast:31
assert_return(() => call($1, "load_at_page_size", []), "memory_grow64.wast:31", 0);

// memory_grow64.wast:32
assert_return(() => call($1, "store_at_page_size", []), "memory_grow64.wast:32");

// memory_grow64.wast:33
assert_return(() => call($1, "load_at_page_size", []), "memory_grow64.wast:33", 3);

// memory_grow64.wast:36
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x86\x80\x80\x80\x00\x01\x60\x01\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x05\x83\x80\x80\x80\x00\x01\x04\x00\x07\x88\x80\x80\x80\x00\x01\x04\x67\x72\x6f\x77\x00\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b", "memory_grow64.wast:36");

// memory_grow64.wast:36
let $2 = instance($$2);

// memory_grow64.wast:41
assert_return(() => call($2, "grow", [0n]), "memory_grow64.wast:41", 0n);

// memory_grow64.wast:42
assert_return(() => call($2, "grow", [1n]), "memory_grow64.wast:42", 0n);

// memory_grow64.wast:43
assert_return(() => call($2, "grow", [0n]), "memory_grow64.wast:43", 1n);

// memory_grow64.wast:44
assert_return(() => call($2, "grow", [2n]), "memory_grow64.wast:44", 1n);

// memory_grow64.wast:45
assert_return(() => call($2, "grow", [800n]), "memory_grow64.wast:45", 3n);

// memory_grow64.wast:46
assert_return(() => call($2, "grow", [1n]), "memory_grow64.wast:46", 803n);

// memory_grow64.wast:48
let $$3 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x86\x80\x80\x80\x00\x01\x60\x01\x7e\x01\x7e\x03\x82\x80\x80\x80\x00\x01\x00\x05\x84\x80\x80\x80\x00\x01\x05\x00\x0a\x07\x88\x80\x80\x80\x00\x01\x04\x67\x72\x6f\x77\x00\x00\x0a\x8c\x80\x80\x80\x00\x01\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b", "memory_grow64.wast:48");

// memory_grow64.wast:48
let $3 = instance($$3);

// memory_grow64.wast:53
assert_return(() => call($3, "grow", [0n]), "memory_grow64.wast:53", 0n);

// memory_grow64.wast:54
assert_return(() => call($3, "grow", [1n]), "memory_grow64.wast:54", 0n);

// memory_grow64.wast:55
assert_return(() => call($3, "grow", [1n]), "memory_grow64.wast:55", 1n);

// memory_grow64.wast:56
assert_return(() => call($3, "grow", [2n]), "memory_grow64.wast:56", 2n);

// memory_grow64.wast:57
assert_return(() => call($3, "grow", [6n]), "memory_grow64.wast:57", 4n);

// memory_grow64.wast:58
assert_return(() => call($3, "grow", [0n]), "memory_grow64.wast:58", 10n);

// memory_grow64.wast:59
assert_return(() => call($3, "grow", [1n]), "memory_grow64.wast:59", -1n);

// memory_grow64.wast:60
assert_return(() => call($3, "grow", [65_536n]), "memory_grow64.wast:60", -1n);

// memory_grow64.wast:64
let $$4 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8c\x80\x80\x80\x00\x02\x60\x01\x7e\x01\x7e\x60\x02\x7e\x7e\x01\x7f\x03\x83\x80\x80\x80\x00\x02\x00\x01\x05\x83\x80\x80\x80\x00\x01\x04\x01\x07\x9c\x80\x80\x80\x00\x02\x04\x67\x72\x6f\x77\x00\x00\x11\x63\x68\x65\x63\x6b\x2d\x6d\x65\x6d\x6f\x72\x79\x2d\x7a\x65\x72\x6f\x00\x01\x0a\xc4\x80\x80\x80\x00\x02\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b\xb3\x80\x80\x80\x00\x01\x01\x7f\x41\x01\x21\x02\x02\x40\x03\x40\x20\x00\x2d\x00\x00\x21\x02\x20\x02\x41\x00\x47\x0d\x01\x20\x00\x20\x01\x5a\x0d\x01\x20\x00\x42\x01\x7c\x21\x00\x20\x00\x20\x01\x58\x0d\x00\x0b\x0b\x20\x02\x0b", "memory_grow64.wast:64");

// memory_grow64.wast:64
let $4 = instance($$4);

// memory_grow64.wast:85
assert_return(() => call($4, "check-memory-zero", [0n, 65_535n]), "memory_grow64.wast:85", 0);

// memory_grow64.wast:86
assert_return(() => call($4, "grow", [1n]), "memory_grow64.wast:86", 1n);

// memory_grow64.wast:87
assert_return(() => call($4, "check-memory-zero", [65_536n, 131_071n]), "memory_grow64.wast:87", 0);

// memory_grow64.wast:88
assert_return(() => call($4, "grow", [1n]), "memory_grow64.wast:88", 2n);

// memory_grow64.wast:89
assert_return(() => call($4, "check-memory-zero", [131_072n, 196_607n]), "memory_grow64.wast:89", 0);

// memory_grow64.wast:90
assert_return(() => call($4, "grow", [1n]), "memory_grow64.wast:90", 3n);

// memory_grow64.wast:91
assert_return(() => call($4, "check-memory-zero", [196_608n, 262_143n]), "memory_grow64.wast:91", 0);

// memory_grow64.wast:92
assert_return(() => call($4, "grow", [1n]), "memory_grow64.wast:92", 4n);

// memory_grow64.wast:93
assert_return(() => call($4, "check-memory-zero", [262_144n, 327_679n]), "memory_grow64.wast:93", 0);

// memory_grow64.wast:94
assert_return(() => call($4, "grow", [1n]), "memory_grow64.wast:94", 5n);

// memory_grow64.wast:95
assert_return(() => call($4, "check-memory-zero", [327_680n, 393_215n]), "memory_grow64.wast:95", 0);
reinitializeRegistry();
})();
