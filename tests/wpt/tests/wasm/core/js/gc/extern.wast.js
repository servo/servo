(function extern_wast_js() {

// extern.wast:1
let $$1 = module("\x00\x61\x73\x6d\x01\x00\x00\x00\x01\xa1\x80\x80\x80\x00\x08\x60\x00\x00\x5f\x00\x5e\x78\x00\x60\x01\x6f\x00\x60\x01\x6f\x01\x6e\x60\x01\x6e\x01\x6f\x60\x01\x7f\x01\x6f\x60\x01\x7f\x01\x6e\x03\x87\x80\x80\x80\x00\x06\x00\x03\x04\x05\x06\x07\x04\x84\x80\x80\x80\x00\x01\x6e\x00\x0a\x06\x8f\x80\x80\x80\x00\x02\x6f\x00\xd0\x6e\xfb\x1b\x0b\x6e\x00\xd0\x6f\xfb\x1a\x0b\x07\xc5\x80\x80\x80\x00\x05\x04\x69\x6e\x69\x74\x00\x01\x0b\x69\x6e\x74\x65\x72\x6e\x61\x6c\x69\x7a\x65\x00\x02\x0b\x65\x78\x74\x65\x72\x6e\x61\x6c\x69\x7a\x65\x00\x03\x0d\x65\x78\x74\x65\x72\x6e\x61\x6c\x69\x7a\x65\x2d\x69\x00\x04\x0e\x65\x78\x74\x65\x72\x6e\x61\x6c\x69\x7a\x65\x2d\x69\x69\x00\x05\x09\x85\x80\x80\x80\x00\x01\x03\x00\x01\x00\x0a\xe7\x80\x80\x80\x00\x06\x82\x80\x80\x80\x00\x00\x0b\xa8\x80\x80\x80\x00\x00\x41\x00\xd0\x6e\x26\x00\x41\x01\x41\x07\xfb\x1c\x26\x00\x41\x02\xfb\x01\x01\x26\x00\x41\x03\x41\x00\xfb\x07\x02\x26\x00\x41\x04\x20\x00\xfb\x1a\x26\x00\x0b\x86\x80\x80\x80\x00\x00\x20\x00\xfb\x1a\x0b\x86\x80\x80\x80\x00\x00\x20\x00\xfb\x1b\x0b\x88\x80\x80\x80\x00\x00\x20\x00\x25\x00\xfb\x1b\x0b\x8a\x80\x80\x80\x00\x00\x20\x00\x25\x00\xfb\x1b\xfb\x1a\x0b", "extern.wast:1");

// extern.wast:1
let $1 = instance($$1);

// extern.wast:37
run(() => call($1, "init", [hostref(0)]), "extern.wast:37");

// extern.wast:39
assert_return(() => call($1, "internalize", [hostref(1)]), "extern.wast:39", hostref(1));

// extern.wast:40
assert_return(() => call($1, "internalize", [null]), "extern.wast:40", null);

// extern.wast:42
assert_return(() => call($1, "externalize", [hostref(2)]), "extern.wast:42", hostref(2));

// extern.wast:43
assert_return(() => call($1, "externalize", [null]), "extern.wast:43", null);

// extern.wast:45
assert_return(() => call($1, "externalize-i", [0]), "extern.wast:45", null);

// extern.wast:46
assert_return(() => call($1, "externalize-i", [1]), "extern.wast:46", "ref.extern");

// extern.wast:47
assert_return(() => call($1, "externalize-i", [2]), "extern.wast:47", "ref.extern");

// extern.wast:48
assert_return(() => call($1, "externalize-i", [3]), "extern.wast:48", "ref.extern");

// extern.wast:49
assert_return(() => call($1, "externalize-i", [4]), "extern.wast:49", "ref.extern");

// extern.wast:50
assert_return(() => call($1, "externalize-i", [5]), "extern.wast:50", null);

// extern.wast:52
assert_return(() => call($1, "externalize-ii", [0]), "extern.wast:52", null);

// extern.wast:53
assert_return(() => call($1, "externalize-ii", [1]), "extern.wast:53", "ref.i31");

// extern.wast:54
assert_return(() => call($1, "externalize-ii", [2]), "extern.wast:54", "ref.struct");

// extern.wast:55
assert_return(() => call($1, "externalize-ii", [3]), "extern.wast:55", "ref.array");

// extern.wast:56
assert_return(() => call($1, "externalize-ii", [4]), "extern.wast:56", hostref(0));

// extern.wast:57
assert_return(() => call($1, "externalize-ii", [5]), "extern.wast:57", null);
reinitializeRegistry();
})();
