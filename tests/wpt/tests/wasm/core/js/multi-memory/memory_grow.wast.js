(function memory_grow_wast_js() {

// memory_grow.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x05\x86\x80\x80\x80\x00\x02\x01\x02\x05\x00\x00\x07\x8f\x80\x80\x80\x00\x02\x04\x6d\x65\x6d\x31\x02\x00\x04\x6d\x65\x6d\x32\x02\x01", "memory_grow.wast:1");

// memory_grow.wast:1
let $1 = instance($$1);

// memory_grow.wast:5
register("M", $1)

// memory_grow.wast:7
let $$2 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x00\x01\x7f\x60\x01\x7f\x01\x7f\x02\x96\x80\x80\x80\x00\x02\x01\x4d\x04\x6d\x65\x6d\x31\x02\x01\x01\x06\x01\x4d\x04\x6d\x65\x6d\x32\x02\x00\x00\x03\x89\x80\x80\x80\x00\x08\x00\x00\x00\x00\x01\x01\x01\x01\x05\x86\x80\x80\x80\x00\x02\x00\x03\x01\x04\x05\x07\xc1\x80\x80\x80\x00\x08\x05\x73\x69\x7a\x65\x31\x00\x00\x05\x73\x69\x7a\x65\x32\x00\x01\x05\x73\x69\x7a\x65\x33\x00\x02\x05\x73\x69\x7a\x65\x34\x00\x03\x05\x67\x72\x6f\x77\x31\x00\x04\x05\x67\x72\x6f\x77\x32\x00\x05\x05\x67\x72\x6f\x77\x33\x00\x06\x05\x67\x72\x6f\x77\x34\x00\x07\x0a\xd1\x80\x80\x80\x00\x08\x84\x80\x80\x80\x00\x00\x3f\x00\x0b\x84\x80\x80\x80\x00\x00\x3f\x01\x0b\x84\x80\x80\x80\x00\x00\x3f\x02\x0b\x84\x80\x80\x80\x00\x00\x3f\x03\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x01\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x02\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x03\x0b", "memory_grow.wast:7");

// memory_grow.wast:7
let $2 = instance($$2);

// memory_grow.wast:32
assert_return(() => call($2, "size1", []), "memory_grow.wast:32", 2);

// memory_grow.wast:33
assert_return(() => call($2, "size2", []), "memory_grow.wast:33", 0);

// memory_grow.wast:34
assert_return(() => call($2, "size3", []), "memory_grow.wast:34", 3);

// memory_grow.wast:35
assert_return(() => call($2, "size4", []), "memory_grow.wast:35", 4);

// memory_grow.wast:37
assert_return(() => call($2, "grow1", [1]), "memory_grow.wast:37", 2);

// memory_grow.wast:38
assert_return(() => call($2, "size1", []), "memory_grow.wast:38", 3);

// memory_grow.wast:39
assert_return(() => call($2, "size2", []), "memory_grow.wast:39", 0);

// memory_grow.wast:40
assert_return(() => call($2, "size3", []), "memory_grow.wast:40", 3);

// memory_grow.wast:41
assert_return(() => call($2, "size4", []), "memory_grow.wast:41", 4);

// memory_grow.wast:43
assert_return(() => call($2, "grow1", [2]), "memory_grow.wast:43", 3);

// memory_grow.wast:44
assert_return(() => call($2, "size1", []), "memory_grow.wast:44", 5);

// memory_grow.wast:45
assert_return(() => call($2, "size2", []), "memory_grow.wast:45", 0);

// memory_grow.wast:46
assert_return(() => call($2, "size3", []), "memory_grow.wast:46", 3);

// memory_grow.wast:47
assert_return(() => call($2, "size4", []), "memory_grow.wast:47", 4);

// memory_grow.wast:49
assert_return(() => call($2, "grow1", [1]), "memory_grow.wast:49", -1);

// memory_grow.wast:50
assert_return(() => call($2, "size1", []), "memory_grow.wast:50", 5);

// memory_grow.wast:51
assert_return(() => call($2, "size2", []), "memory_grow.wast:51", 0);

// memory_grow.wast:52
assert_return(() => call($2, "size3", []), "memory_grow.wast:52", 3);

// memory_grow.wast:53
assert_return(() => call($2, "size4", []), "memory_grow.wast:53", 4);

// memory_grow.wast:55
assert_return(() => call($2, "grow2", [10]), "memory_grow.wast:55", 0);

// memory_grow.wast:56
assert_return(() => call($2, "size1", []), "memory_grow.wast:56", 5);

// memory_grow.wast:57
assert_return(() => call($2, "size2", []), "memory_grow.wast:57", 10);

// memory_grow.wast:58
assert_return(() => call($2, "size3", []), "memory_grow.wast:58", 3);

// memory_grow.wast:59
assert_return(() => call($2, "size4", []), "memory_grow.wast:59", 4);

// memory_grow.wast:61
assert_return(() => call($2, "grow3", [268_435_456]), "memory_grow.wast:61", -1);

// memory_grow.wast:62
assert_return(() => call($2, "size1", []), "memory_grow.wast:62", 5);

// memory_grow.wast:63
assert_return(() => call($2, "size2", []), "memory_grow.wast:63", 10);

// memory_grow.wast:64
assert_return(() => call($2, "size3", []), "memory_grow.wast:64", 3);

// memory_grow.wast:65
assert_return(() => call($2, "size4", []), "memory_grow.wast:65", 4);

// memory_grow.wast:67
assert_return(() => call($2, "grow3", [3]), "memory_grow.wast:67", 3);

// memory_grow.wast:68
assert_return(() => call($2, "size1", []), "memory_grow.wast:68", 5);

// memory_grow.wast:69
assert_return(() => call($2, "size2", []), "memory_grow.wast:69", 10);

// memory_grow.wast:70
assert_return(() => call($2, "size3", []), "memory_grow.wast:70", 6);

// memory_grow.wast:71
assert_return(() => call($2, "size4", []), "memory_grow.wast:71", 4);

// memory_grow.wast:73
assert_return(() => call($2, "grow4", [1]), "memory_grow.wast:73", 4);

// memory_grow.wast:74
assert_return(() => call($2, "grow4", [1]), "memory_grow.wast:74", -1);

// memory_grow.wast:75
assert_return(() => call($2, "size1", []), "memory_grow.wast:75", 5);

// memory_grow.wast:76
assert_return(() => call($2, "size2", []), "memory_grow.wast:76", 10);

// memory_grow.wast:77
assert_return(() => call($2, "size3", []), "memory_grow.wast:77", 6);

// memory_grow.wast:78
assert_return(() => call($2, "size4", []), "memory_grow.wast:78", 5);

// memory_grow.wast:81
let $$3 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\x8a\x80\x80\x80\x00\x02\x60\x01\x7f\x01\x7f\x60\x00\x01\x7f\x03\x85\x80\x80\x80\x00\x04\x00\x00\x01\x01\x05\x85\x80\x80\x80\x00\x02\x00\x01\x00\x02\x07\xa1\x80\x80\x80\x00\x04\x05\x67\x72\x6f\x77\x31\x00\x00\x05\x67\x72\x6f\x77\x32\x00\x01\x05\x73\x69\x7a\x65\x31\x00\x02\x05\x73\x69\x7a\x65\x32\x00\x03\x0a\xa9\x80\x80\x80\x00\x04\x86\x80\x80\x80\x00\x00\x20\x00\x40\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\x40\x01\x0b\x84\x80\x80\x80\x00\x00\x3f\x00\x0b\x84\x80\x80\x80\x00\x00\x3f\x01\x0b", "memory_grow.wast:81");

// memory_grow.wast:81
let $3 = instance($$3);

// memory_grow.wast:96
assert_return(() => call($3, "size1", []), "memory_grow.wast:96", 1);

// memory_grow.wast:97
assert_return(() => call($3, "size2", []), "memory_grow.wast:97", 2);

// memory_grow.wast:98
assert_return(() => call($3, "grow1", [3]), "memory_grow.wast:98", 1);

// memory_grow.wast:99
assert_return(() => call($3, "grow1", [4]), "memory_grow.wast:99", 4);

// memory_grow.wast:100
assert_return(() => call($3, "grow1", [1]), "memory_grow.wast:100", 8);

// memory_grow.wast:101
assert_return(() => call($3, "grow2", [1]), "memory_grow.wast:101", 2);

// memory_grow.wast:102
assert_return(() => call($3, "grow2", [1]), "memory_grow.wast:102", 3);
reinitializeRegistry();
})();
