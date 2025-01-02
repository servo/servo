'use strict';

function test_query_selector(parentNode, selector, expected) {
  if (!Array.isArray(expected))
    expected = [ expected ];

  test(function(){
    const elementList = parentNode.querySelectorAll(selector);
    assert_equals(elementList.length, expected.length);

    for (let i = 0; i < elementList.length; ++i) {
      if (typeof expected[i] === 'string')
        assert_equals(elementList[i].id, expected[i]);
      else
        assert_equals(elementList[i], expected[i]);
    }
  }, "Selector '" + selector + '" should find the expected elements');
}
