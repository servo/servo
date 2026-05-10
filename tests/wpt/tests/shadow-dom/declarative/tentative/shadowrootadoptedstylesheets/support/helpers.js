// Helper utilities for shadowrootadoptedstylesheets tests.

// Creates a shadow root host via setHTMLUnsafe with the given specifiers
// and returns { host, shadowRoot, testElement }.
function createStylesheetHost(specifiers, hostId = "host", elementId = "test_element") {
  const attr = Array.isArray(specifiers) ? specifiers.join(" ") : specifiers;
  document.body.setHTMLUnsafe(
    `<div id='${hostId}'>` +
      `<template shadowrootmode='open' shadowrootadoptedstylesheets='${attr}'>` +
        `<span id='${elementId}'>Test content</span>` +
      "</template>" +
    "</div>"
  );
  const host = document.getElementById(hostId);
  const shadowRoot = host.shadowRoot;
  const testElement = shadowRoot.getElementById(elementId);
  return { host, shadowRoot, testElement };
}

// Asserts that adoptedStyleSheets[index] has a single rule matching the
// expected cssText.
function assertSheetRule(shadowRoot, index, expectedCssText, description) {
  assert_equals(
    shadowRoot.adoptedStyleSheets[index].cssRules.length,
    1,
    `${description}: sheet at index ${index} should have one rule.`,
  );
  assert_equals(
    shadowRoot.adoptedStyleSheets[index].cssRules[0].cssText,
    expectedCssText,
    `${description}: sheet at index ${index} rule text.`,
  );
}

// Awaits the import of the given URL(s) and waits for the adopted stylesheet
// updates to be applied. Two rAF waits are needed to ensure the task queue is
// drained. TODO: Remove double rAF when placeholders are dropped.
async function fetchAndWait(...urls) {
  await Promise.all(
    urls.map(url => import(url, { with: { type: "css" } }).catch(() => {}))
  );
  await new Promise(resolve => requestAnimationFrame(resolve));
  await new Promise(resolve => requestAnimationFrame(resolve));
}
