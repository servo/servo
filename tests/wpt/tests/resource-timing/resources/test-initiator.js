if (observe_entry === undefined) {
  throw new Error("You must include resource-timing/resources/observe-entry.js "
    + "before including this script.");
}

// Asserts that, for the given name, there is/will-be a
// PerformanceResourceTiming entry that has the given 'initiatorUrl'. The test
// is labeled according to the given descriptor.
const initiator_url_test = (resourceName, expectedUrl, descriptor) => {
  promise_test(async () => {
    const entry = await observe_entry(resourceName);
    assert_equals(entry.initiatorUrl, expectedUrl);
  }, `The initiator Url for ${descriptor} must be '${expectedUrl}'`);
};

const initiator_url_doc_write_test = (write_to_doc,
  resourceName, expectedUrl, descriptor) => {
  promise_test(async () => {
    document.open();
    document.write(write_to_doc);
    document.close();
    const entry = await observe_entry(resourceName);
    assert_equals(entry.initiatorUrl, expectedUrl);
  }, `The initiator Url for ${descriptor} must be '${expectedUrl}'`);
};
