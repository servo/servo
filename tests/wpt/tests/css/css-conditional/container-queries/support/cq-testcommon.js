function assert_implements_container_queries() {
  assert_implements(CSS.supports("container-type:size"), "Basic support for container queries required");
}

function cleanup_container_query_main() {
  const main = document.querySelector("#cq-main");
  while (main.firstChild)
    main.firstChild.remove();
}

function set_container_query_style(text) {
  let style = document.createElement('style');
  style.innerText = text;
  document.querySelector("#cq-main").append(style);
  return style;
}

function test_cq_rule_valid(query) {
  test(t => {
    t.add_cleanup(cleanup_container_query_main);
    let style = set_container_query_style(`@container ${query} {}`);
    assert_equals(style.sheet.rules.length, 1);
  }, query);
}

function test_cq_condition_invalid(condition) {
  test(t => {
    t.add_cleanup(cleanup_container_query_main);
    let style = set_container_query_style(`@container name ${condition} {}`);
    assert_equals(style.sheet.rules.length, 0);
  }, condition);
}

// Tests that 1) the condition parses, and 2) is either "unknown" or not, as
// specified.
function test_cq_condition_valid(condition, unknown) {
  test(t => {
    t.add_cleanup(cleanup_container_query_main);
    let style = set_container_query_style(`
      @container name ${condition} {}
      @container name (${condition}) or (not (${condition})) { main { --match:true; } }
    `);
    assert_equals(style.sheet.rules.length, 2);
    const expected = unknown ? '' : 'true';
    assert_equals(getComputedStyle(document.querySelector("#cq-main")).getPropertyValue('--match'), expected);
  }, condition);
}

function test_cq_condition_known(condition) {
  test_cq_condition_valid(condition, false /* unknown */);
}

function test_cq_condition_unknown(condition) {
  test_cq_condition_valid(condition, true /* unknown */);
}

function test_container_name_invalid(container_name) {
  test(t => {
    t.add_cleanup(cleanup_container_query_main);
    let style = set_container_query_style(`@container ${container_name} not (width) {}`);
    assert_equals(style.sheet.rules.length, 0);
  }, `Container name: ${container_name}`);
}

function test_container_name_valid(container_name) {
  test(t => {
    t.add_cleanup(cleanup_container_query_main);
    let style = set_container_query_style(`@container ${container_name} not (width) {}`);
    assert_equals(style.sheet.rules.length, 1);
  }, `Container name: ${container_name}`);
}
