(function table_size64_wast_js() {

// table_size64.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x01\x7e\x60\x01\x7e\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x01\x01\x01\x01\x04\x8f\x80\x80\x80\x00\x04\x6f\x04\x00\x6f\x04\x01\x6f\x05\x00\x02\x6f\x05\x03\x08\x07\xd1\x80\x80\x80\x00\x08\x07\x73\x69\x7a\x65\x2d\x74\x30\x00\x00\x07\x73\x69\x7a\x65\x2d\x74\x31\x00\x01\x07\x73\x69\x7a\x65\x2d\x74\x32\x00\x02\x07\x73\x69\x7a\x65\x2d\x74\x33\x00\x03\x07\x67\x72\x6f\x77\x2d\x74\x30\x00\x04\x07\x67\x72\x6f\x77\x2d\x74\x31\x00\x05\x07\x67\x72\x6f\x77\x2d\x74\x32\x00\x06\x07\x67\x72\x6f\x77\x2d\x74\x33\x00\x07\x0a\xe5\x80\x80\x80\x00\x08\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x01\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x02\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x03\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x00\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x01\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x02\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x03\x1a\x0b", "table_size64.wast:1");

// table_size64.wast:1
let $1 = instance($$1);

// table_size64.wast:26
assert_return(() => call($1, "size-t0", []), "table_size64.wast:26", 0n);

// table_size64.wast:27
assert_return(() => call($1, "grow-t0", [1n]), "table_size64.wast:27");

// table_size64.wast:28
assert_return(() => call($1, "size-t0", []), "table_size64.wast:28", 1n);

// table_size64.wast:29
assert_return(() => call($1, "grow-t0", [4n]), "table_size64.wast:29");

// table_size64.wast:30
assert_return(() => call($1, "size-t0", []), "table_size64.wast:30", 5n);

// table_size64.wast:31
assert_return(() => call($1, "grow-t0", [0n]), "table_size64.wast:31");

// table_size64.wast:32
assert_return(() => call($1, "size-t0", []), "table_size64.wast:32", 5n);

// table_size64.wast:34
assert_return(() => call($1, "size-t1", []), "table_size64.wast:34", 1n);

// table_size64.wast:35
assert_return(() => call($1, "grow-t1", [1n]), "table_size64.wast:35");

// table_size64.wast:36
assert_return(() => call($1, "size-t1", []), "table_size64.wast:36", 2n);

// table_size64.wast:37
assert_return(() => call($1, "grow-t1", [4n]), "table_size64.wast:37");

// table_size64.wast:38
assert_return(() => call($1, "size-t1", []), "table_size64.wast:38", 6n);

// table_size64.wast:39
assert_return(() => call($1, "grow-t1", [0n]), "table_size64.wast:39");

// table_size64.wast:40
assert_return(() => call($1, "size-t1", []), "table_size64.wast:40", 6n);

// table_size64.wast:42
assert_return(() => call($1, "size-t2", []), "table_size64.wast:42", 0n);

// table_size64.wast:43
assert_return(() => call($1, "grow-t2", [3n]), "table_size64.wast:43");

// table_size64.wast:44
assert_return(() => call($1, "size-t2", []), "table_size64.wast:44", 0n);

// table_size64.wast:45
assert_return(() => call($1, "grow-t2", [1n]), "table_size64.wast:45");

// table_size64.wast:46
assert_return(() => call($1, "size-t2", []), "table_size64.wast:46", 1n);

// table_size64.wast:47
assert_return(() => call($1, "grow-t2", [0n]), "table_size64.wast:47");

// table_size64.wast:48
assert_return(() => call($1, "size-t2", []), "table_size64.wast:48", 1n);

// table_size64.wast:49
assert_return(() => call($1, "grow-t2", [4n]), "table_size64.wast:49");

// table_size64.wast:50
assert_return(() => call($1, "size-t2", []), "table_size64.wast:50", 1n);

// table_size64.wast:51
assert_return(() => call($1, "grow-t2", [1n]), "table_size64.wast:51");

// table_size64.wast:52
assert_return(() => call($1, "size-t2", []), "table_size64.wast:52", 2n);

// table_size64.wast:54
assert_return(() => call($1, "size-t3", []), "table_size64.wast:54", 3n);

// table_size64.wast:55
assert_return(() => call($1, "grow-t3", [1n]), "table_size64.wast:55");

// table_size64.wast:56
assert_return(() => call($1, "size-t3", []), "table_size64.wast:56", 4n);

// table_size64.wast:57
assert_return(() => call($1, "grow-t3", [3n]), "table_size64.wast:57");

// table_size64.wast:58
assert_return(() => call($1, "size-t3", []), "table_size64.wast:58", 7n);

// table_size64.wast:59
assert_return(() => call($1, "grow-t3", [0n]), "table_size64.wast:59");

// table_size64.wast:60
assert_return(() => call($1, "size-t3", []), "table_size64.wast:60", 7n);

// table_size64.wast:61
assert_return(() => call($1, "grow-t3", [2n]), "table_size64.wast:61");

// table_size64.wast:62
assert_return(() => call($1, "size-t3", []), "table_size64.wast:62", 7n);

// table_size64.wast:63
assert_return(() => call($1, "grow-t3", [1n]), "table_size64.wast:63");

// table_size64.wast:64
assert_return(() => call($1, "size-t3", []), "table_size64.wast:64", 8n);
reinitializeRegistry();
})();
