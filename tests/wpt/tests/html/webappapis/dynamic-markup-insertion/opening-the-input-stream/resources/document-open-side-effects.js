function assertDocumentIsReadyForSideEffectsTest(doc, description) {
  assert_not_equals(doc.childNodes.length, 0, `document should not be empty before side effects test (${description})`);
}

function assertOpenHasNoSideEffects(doc, originalURL, description) {
  assert_not_equals(doc.childNodes.length, 0, `document nodes should not be cleared (${description})`);
  assert_equals(doc.URL, originalURL, `The original URL should be kept (${description})`);
}
