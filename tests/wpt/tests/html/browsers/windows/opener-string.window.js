test(t => {
  const popup = window.open();
  t.add_cleanup(() => popup.close());
  assert_equals(popup.opener, self, "The opener of the popup is me");
  assert_equals(Object.getOwnPropertyDescriptor(popup, "opener").writable, undefined);

  popup.opener = "blah";
  assert_equals(popup.opener, "blah", "The popup's opener is now a string");
  assert_equals(Object.getOwnPropertyDescriptor(popup, "opener").writable, true);

  const openerGetter = Object.getOwnPropertyDescriptor(self, "opener").get;
  const popupOpener = openerGetter.call(popup);
  assert_equals(popupOpener, self, "The underlying opener of the popup is still me");
}, "Setting popup.opener to a string");
