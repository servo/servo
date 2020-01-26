test(() => {
  assert_throws_js(TypeError, () => {
    new Request("", {importance: 'invalid'});
  }, "a new Request() must throw a TypeError if RequestInit's importance is an invalid value");
}, "new Request() throws a TypeError if any of RequestInit's members' values are invalid");
