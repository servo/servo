// META: title=FormData: constructor

test(() => {
  assert_throws_js(TypeError, () => { new FormData(null); });
  assert_throws_js(TypeError, () => { new FormData("string"); });
}, "Constructors should throw a type error");
