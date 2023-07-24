/* Utilities related to WAI-ARIA */

const AriaUtils = {

  /*
  Tests simple role assignment: <div role="alert">x</div>
  Not intended for nested, context-dependent, or other complex role tests.

  Ex: AriaUtils.assignAndVerifyRolesByRoleNames(["group", "main", "button"])

  */
  assignAndVerifyRolesByRoleNames: function(roleNames) {
    if (!Array.isArray(roleNames) || !roleNames.length) {
      throw `Param roleNames of assignAndVerifyRolesByRoleNames("${roleNames}") should be an array containing at least one role string.`;
    }
    for (const role of roleNames) {
      promise_test(async t => {
        let el = document.createElement("div");
        el.appendChild(document.createTextNode("x"));
        el.setAttribute("role", role); // el.role not yet supported by Gecko.
        document.body.appendChild(el);
        const computedRole = await test_driver.get_computed_role(el);
        assert_equals(computedRole, role, el.outerHTML);
      }, `role: ${role}`);
    }
  },


  /*
  Tests computed ROLE of all elements matching selector
  against the string value of their data-expectedrole attribute.

  Ex: <div role="list"
        data-testname="optional unique test name"
        data-expectedrole="list"
        class="ex">

      AriaUtils.verifyRolesBySelector(".ex")

  */
  verifyRolesBySelector: function(selector) {
    const els = document.querySelectorAll(selector);
    if (!els.length) {
      throw `Selector passed in verifyRolesBySelector("${selector}") should match at least one element.`;
    }
    for (const el of els) {
      let role = el.getAttribute("data-expectedrole");
      let testName = el.getAttribute("data-testname") || role; // data-testname optional if role is unique per test file
      promise_test(async t => {
        const expectedRole = el.getAttribute("data-expectedrole");
        const computedRole = await test_driver.get_computed_role(el);
        assert_equals(computedRole, expectedRole, el.outerHTML);
      }, `${testName}`);
    }
  },


  /*
  Tests computed LABEL of all elements matching selector
  against the string value of their data-expectedlabel attribute.

  Ex: <div aria-label="foo"
        data-testname="optional unique test name"
        data-expectedlabel="foo"
        class="ex">

      AriaUtils.verifyLabelsBySelector(".ex")

  */
  verifyLabelsBySelector: function(selector) {
    const els = document.querySelectorAll(selector);
    if (!els.length) {
      throw `Selector passed in verifyLabelsBySelector("${selector}") should match at least one element.`;
    }
    for (const el of els) {
      let label = el.getAttribute("data-expectedlabel");
      let testName = el.getAttribute("data-testname") || label; // data-testname optional if label is unique per test file
      promise_test(async t => {
        const expectedLabel = el.getAttribute("data-expectedlabel");
        let computedLabel = await test_driver.get_computed_label(el);
        // Todo: Remove whitespace normalization after https://github.com/w3c/accname/issues/192 is addressed. Change prior line back to `const`, too.
        computedLabel = computedLabel.trim()
        assert_equals(computedLabel, expectedLabel, el.outerHTML);
      }, `${testName}`);
    }
  },


};

