[
  "registerContentHandler",
  "isProtocolHandlerRegistered",
  "isContentHandlerRegistered",
  "unregisterContentHandler"
].forEach(method => {
  test(() => {
    assert_false(method in self.navigator);
  }, method + "() is removed");
});
