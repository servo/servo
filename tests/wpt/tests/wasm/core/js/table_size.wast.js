(function table_size_wast_js() {

// table_size.wast:1
let $1 = instance("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x89\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x01\x7f\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x01\x01\x01\x01\x04\x8f\x80\x80\x80\x00\x04\x6f\x00\x00\x6f\x00\x01\x6f\x01\x00\x02\x6f\x01\x03\x08\x07\xd1\x80\x80\x80\x00\x08\x07\x73\x69\x7a\x65\x2d\x74\x30\x00\x00\x07\x73\x69\x7a\x65\x2d\x74\x31\x00\x01\x07\x73\x69\x7a\x65\x2d\x74\x32\x00\x02\x07\x73\x69\x7a\x65\x2d\x74\x33\x00\x03\x07\x67\x72\x6f\x77\x2d\x74\x30\x00\x04\x07\x67\x72\x6f\x77\x2d\x74\x31\x00\x05\x07\x67\x72\x6f\x77\x2d\x74\x32\x00\x06\x07\x67\x72\x6f\x77\x2d\x74\x33\x00\x07\x0a\xe5\x80\x80\x80\x00\x08\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x01\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x02\x0b\x85\x80\x80\x80\x00\x00\xfc\x10\x03\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x00\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x01\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x02\x1a\x0b\x8a\x80\x80\x80\x00\x00\xd0\x6f\x20\x00\xfc\x0f\x03\x1a\x0b");

// table_size.wast:26
assert_return(() => call($1, "size-t0", []), 0);

// table_size.wast:27
assert_return(() => call($1, "grow-t0", [1]));

// table_size.wast:28
assert_return(() => call($1, "size-t0", []), 1);

// table_size.wast:29
assert_return(() => call($1, "grow-t0", [4]));

// table_size.wast:30
assert_return(() => call($1, "size-t0", []), 5);

// table_size.wast:31
assert_return(() => call($1, "grow-t0", [0]));

// table_size.wast:32
assert_return(() => call($1, "size-t0", []), 5);

// table_size.wast:34
assert_return(() => call($1, "size-t1", []), 1);

// table_size.wast:35
assert_return(() => call($1, "grow-t1", [1]));

// table_size.wast:36
assert_return(() => call($1, "size-t1", []), 2);

// table_size.wast:37
assert_return(() => call($1, "grow-t1", [4]));

// table_size.wast:38
assert_return(() => call($1, "size-t1", []), 6);

// table_size.wast:39
assert_return(() => call($1, "grow-t1", [0]));

// table_size.wast:40
assert_return(() => call($1, "size-t1", []), 6);

// table_size.wast:42
assert_return(() => call($1, "size-t2", []), 0);

// table_size.wast:43
assert_return(() => call($1, "grow-t2", [3]));

// table_size.wast:44
assert_return(() => call($1, "size-t2", []), 0);

// table_size.wast:45
assert_return(() => call($1, "grow-t2", [1]));

// table_size.wast:46
assert_return(() => call($1, "size-t2", []), 1);

// table_size.wast:47
assert_return(() => call($1, "grow-t2", [0]));

// table_size.wast:48
assert_return(() => call($1, "size-t2", []), 1);

// table_size.wast:49
assert_return(() => call($1, "grow-t2", [4]));

// table_size.wast:50
assert_return(() => call($1, "size-t2", []), 1);

// table_size.wast:51
assert_return(() => call($1, "grow-t2", [1]));

// table_size.wast:52
assert_return(() => call($1, "size-t2", []), 2);

// table_size.wast:54
assert_return(() => call($1, "size-t3", []), 3);

// table_size.wast:55
assert_return(() => call($1, "grow-t3", [1]));

// table_size.wast:56
assert_return(() => call($1, "size-t3", []), 4);

// table_size.wast:57
assert_return(() => call($1, "grow-t3", [3]));

// table_size.wast:58
assert_return(() => call($1, "size-t3", []), 7);

// table_size.wast:59
assert_return(() => call($1, "grow-t3", [0]));

// table_size.wast:60
assert_return(() => call($1, "size-t3", []), 7);

// table_size.wast:61
assert_return(() => call($1, "grow-t3", [2]));

// table_size.wast:62
assert_return(() => call($1, "size-t3", []), 7);

// table_size.wast:63
assert_return(() => call($1, "grow-t3", [1]));

// table_size.wast:64
assert_return(() => call($1, "size-t3", []), 8);

// table_size.wast:69
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x84\x80\x80\x80\x00\x01\x60\x00\x00\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x01\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b");

// table_size.wast:78
assert_invalid("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x85\x80\x80\x80\x00\x01\x60\x00\x01\x7d\x03\x82\x80\x80\x80\x00\x01\x00\x04\x84\x80\x80\x80\x00\x01\x6f\x00\x01\x0a\x8b\x80\x80\x80\x00\x01\x85\x80\x80\x80\x00\x00\xfc\x10\x00\x0b");
reinitializeRegistry();
})();
