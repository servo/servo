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
// Elements <div id=target> and <div=main> are assumed to exist.
function test_all_templates() {
  let templates = document.querySelectorAll('template');
  for (let template of templates) {
    test((t) => {
      t.add_cleanup(() => main.replaceChildren());
      main.append(template.content.cloneNode(true));
      let cs = getComputedStyle(target);
      let actual = cs.getPropertyValue('--actual');
      let expected = cs.getPropertyValue('--expected');
      assert_equals(actual, expected);
    }, template.getAttribute('data-name'));
  }
}
