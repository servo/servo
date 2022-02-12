promise_test(async () => {
  assert_equals(resources_script_result, 'loaded from webbundle');
  assert_equals(scopes_script_result, 'loaded from webbundle');
  assert_equals(out_of_scope_script_result, 'loaded from network');

  ['resources_', 'scopes_'].forEach((type) => {
    ['style_target',
      'style_imported_from_file_target',
      'style_imported_from_tag_target'].forEach((target) => {
        const element = document.createElement('div');
        element.id = type + target;
        document.body.appendChild(element);
        assert_equals(window.getComputedStyle(element).color,
                      'rgb(0, 0, 255)',
                      element.id + ' color must be blue');
    });
  });
}, 'Subresources from static elements should be loaded from web bundle.');