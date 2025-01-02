function init(e, method) {
  /*
   * This test suite tests Selectors API methods in 4 different contexts:
   * 1. Document node
   * 2. In-document Element node
   * 3. Detached Element node (an element with no parent, not in the document)
   * 4. Document Fragment node
   *
   * For each context, the following tests are run:
   *
   * The interface check tests ensure that each type of node exposes the Selectors API methods.
   *
   * The matches() tests are run
   * All the selectors tested for both the valid and invalid selector tests are found in selectors.js.
   * See comments in that file for documentation of the format used.
   *
   * The level2-lib.js file contains all the common test functions for running each of the aforementioned tests
   */

  var docType  = "html"; // Only run tests suitable for HTML

  // Prepare the nodes for testing
  var doc = e.target.contentDocument;                 // Document Node tests

  var element = doc.getElementById("root");   // In-document Element Node tests

  //Setup the namespace tests
  setupSpecialElements(doc, element);

  var outOfScope = element.cloneNode(true);   // Append this to the body before running the in-document
                                               // Element tests, but after running the Document tests. This
                                               // tests that no elements that are not descendants of element
                                               // are selected.

  traverse(outOfScope, function(elem) {        // Annotate each element as being a clone; used for verifying
    elem.setAttribute("data-clone", "");     // that none of these elements ever match.
  });


  var detached = element.cloneNode(true);     // Detached Element Node tests

  var fragment = doc.createDocumentFragment(); // Fragment Node tests
  fragment.appendChild(element.cloneNode(true));

  // Setup Tests
  interfaceCheckMatches(method, "Document", doc);
  interfaceCheckMatches(method, "Detached Element", detached);
  interfaceCheckMatches(method, "Fragment", fragment);
  interfaceCheckMatches(method, "In-document Element", element);

  runSpecialMatchesTests(method, "DIV Element", element);
  runSpecialMatchesTests(method, "NULL Element", document.createElement("null"));
  runSpecialMatchesTests(method, "UNDEFINED Element", document.createElement("undefined"));

  runInvalidSelectorTestMatches(method, "Document", doc, invalidSelectors);
  runInvalidSelectorTestMatches(method, "Detached Element", detached, invalidSelectors);
  runInvalidSelectorTestMatches(method, "Fragment", fragment, invalidSelectors);
  runInvalidSelectorTestMatches(method, "In-document Element", element, invalidSelectors);

  runMatchesTest(method, "In-document", doc, validSelectors, "html");
  runMatchesTest(method, "Detached", detached, validSelectors, "html");
  runMatchesTest(method, "Fragment", fragment, validSelectors, "html");

  runMatchesTest(method, "In-document", doc, scopedSelectors, "html");
}
