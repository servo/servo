test(() => {
  assert_false("getAll" in new Headers)
  assert_false("getAll" in Headers.prototype)
}, "Headers object no longer has a getAll() method")
