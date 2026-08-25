'use strict';
/**
 * Helper functions for speculation rules BiDi testdriver tests
 */
/**
 * Waits until the document has finished loading.
 * @returns {Promise<void>} Resolves if the document is already completely
 *     loaded or when the 'onload' event is fired.
 */
function waitForDocumentReady() {
  return new Promise(resolve => {
    if (document.readyState === 'complete') {
      resolve();
    }
    window.addEventListener('load', () => {
      resolve();
    }, {once: true});
  });
}
/**
 * Adds speculation rules and a corresponding link to the page.
 * @param {Object} speculationRules - The speculation rules object to add
 * @param {string} targetUrl - The URL to add as a link
 * @param {string} linkText - The text content of the link (optional)
 * @returns {Object} Object containing the created script and link elements
 */
function addSpeculationRulesAndLink(speculationRules, targetUrl, linkText = 'Test Link') {
  // Add speculation rules script exactly like the working test
  const script = document.createElement('script');
  script.type = 'speculationrules';
  script.textContent = JSON.stringify(speculationRules);
  document.head.appendChild(script);
  // Also add a link to the page (some implementations might need this)
  const link = document.createElement('a');
  link.href = targetUrl;
  link.textContent = linkText;
  document.body.appendChild(link);
  return { script, link };
}