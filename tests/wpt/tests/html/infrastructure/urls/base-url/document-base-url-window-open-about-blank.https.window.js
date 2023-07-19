// Basic test that a popup about:blank window inherits its base url from
// the initiator (which in this case is also the opener).
const runTest = (description) => {
  // In this test the opener and the initiator will be the same.
  const initiator_base_uri = document.baseURI;
  test(() => {
    const popup = window.open();

    // Window.open synchronously loads the initial empty document.
    assert_equals("about:blank", popup.location.href);
    assert_equals(initiator_base_uri, popup.document.baseURI);

    // Verify the popup's base url is properly snapshotted, and doesn't change
    // if the parent's base url changes.
    const base_element = document.createElement('base');
    base_element.href = "https://example.com";
    document.head.appendChild(base_element);
    assert_equals(initiator_base_uri, popup.document.baseURI);
  }, description);
};

onload = () => {
  runTest("window.open() gets base url from initiator.");
};
