function getElementsByIds(ids) {
  var result = [];
  ids.forEach(function(id) {
    result.push(document.getElementById(id));
  });
  return result;
}

function testSelector(selector, expected, testName) {
  test(function(){
    var elements = document.querySelectorAll(selector);
    assert_array_equals(elements, getElementsByIds(expected));
  }, testName);
}
