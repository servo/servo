test(() => {
  assert_false("showModalDialog" in window)
  assert_false("showModalDialog" in Window.prototype)
}, "showModalDialog() has been removed from the platform")
