var beforehref = location.href;

test(function () {
  history.pushState('','','/testing_ignore_me_404');
  assert_equals(location.href,beforehref.replace(/^(\w*:\/\/[^\/]*\/)[\w\W]*$/,'$1testing_ignore_me_404'));
}, 'history.pushState URL resolving should be done relative to the document, not the script');

test(function () {
  history.replaceState('','','/testing_ignore_me_404_2');
  assert_equals(location.href,beforehref.replace(/^(\w*:\/\/[^\/]*\/)[\w\W]*$/,'$1testing_ignore_me_404_2'));
}, 'history.replaceState URL resolving should be done relative to the document, not the script');
