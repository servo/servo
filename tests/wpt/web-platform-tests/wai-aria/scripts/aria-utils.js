/* Utilities related to WAI-ARIA */

const AriaUtils = {

  /*
  Tests simple role assignment: <div role="alert">x</div>
  Not intended for nested, context-dependent, or other complex role tests.
  */
  assignAndVerifyRolesByRoleNames: function(roleNames) {
    for (const role of roleNames) {
      promise_test(async t => {
        let el = document.createElement("div");
        el.appendChild(document.createTextNode("x"));
        el.setAttribute("role", role); // el.role not yet supported by Gecko.
        el.id = `role_${role}`;
        document.body.appendChild(el);
        const computedRole = await test_driver.get_computed_role(document.getElementById(el.id));
        assert_equals(computedRole, role, el.outerHTML);
      }, `role: ${role}`);
    }
  },

  /*
  Tests computed role of all elements matching selector
  against the string value of their data-role attribute.

  Ex: <div role="list"
        data-testname="optional unique test name"
        data-expectedrole="list"
        class="ex">

      AriaUtils.verifyRolesBySelector(".ex")

  */
  verifyRolesBySelector: function(selector) {
    const els = document.querySelectorAll(selector);
    for (const el of els) {
      let role = el.getAttribute("data-expectedrole");
      let testName = el.getAttribute("data-testname") || role; // data-testname optional if role unique per test file
      promise_test(async t => {
        const expectedRole = el.getAttribute("data-expectedrole");

        // ensure ID existence and uniqueness for the webdriver callback
        if (!el.id) {
          let roleCount = 1;
          let elID = `${expectedRole}${roleCount}`;
          while(document.getElementById(elID)) {
            roleCount++;
            elID = `${expectedRole}${roleCount}`;
          }
          el.id = elID;
        }

        const computedRole = await test_driver.get_computed_role(document.getElementById(el.id));
        assert_equals(computedRole, expectedRole, el.outerHTML);
      }, `${testName}`);
    }
  },

};

