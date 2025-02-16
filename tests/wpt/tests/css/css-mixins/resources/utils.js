// Looks for <template> elements, and runs each test in turn, for example:
//
//  <template data-name="Example test">
//    <style>
//      @function --f() returns <length> {
//        result: calc(1px + 1px);
//      }
//      #target {
//        --actual: --f();
//        --expected: 2px;
//      }
//    </style>
//  </template>
//
// The test passes if the computed value of --actual matches
// the computed value of --expected on #target.
//
// Arguments:
// * `styleTarget`, defaults to <div id=target>, which is assumed to exist.
// * `templateTarget` defaults to <div=main>, which are assumed to exist.
// * `descriptor` optional test descriptor
function test_all_templates(styleTarget = target, templateTarget = main, descriptor = '') {
  let templates = document.querySelectorAll('template');
  for (let template of templates) {
    test((t) => {
      t.add_cleanup(() => templateTarget.replaceChildren());
      templateTarget.append(template.content.cloneNode(true));
      let cs = getComputedStyle(styleTarget);
      let actual = cs.getPropertyValue('--actual');
      let expected = cs.getPropertyValue('--expected');
      assert_equals(actual, expected);
    }, `${descriptor ? `${descriptor}: `: ''}${template.getAttribute('data-name')}`);
  }
}
