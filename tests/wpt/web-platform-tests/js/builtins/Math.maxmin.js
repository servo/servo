function testMathMaxMin(aFun) {
  var test_error = { name: "test" };
  test(function() {
    assert_throws(test_error, function() {
      Math[aFun](NaN, {
        valueOf: function() {
          throw test_error;
        }
      });
    });
  }, "ToNumber should be called on all arguments: NaN.");
  test(function() {
    assert_throws(test_error, function() {
      Math[aFun](-Infinity, {
        valueOf: function() {
          throw test_error;
        }
      });
    });
  }, "ToNumber should be called on all arguments: -Infinity.");
  test(function() {
    assert_throws(test_error, function() {
      Math[aFun](Infinity, {
        valueOf: function() {
          throw test_error;
        }
      });
    });
  }, "ToNumber should be called on all arguments: Infinity.");
  test(function() {
    assert_throws(test_error, function() {
      Math[aFun]({
        valueOf: function() {
          throw test_error;
        }
      },
      {
        valueOf: function() {
          throw 7;
        }
      });
    });
  }, "ToNumber should be called left to right.");
  test(function() {
    assert_equals(Math[aFun]("1"), 1);
  }, "Should return a number.");
  test(function() {
    var expected = {
      "max": 0,
      "min": -0
    }
    assert_equals(Math[aFun](0, -0), expected[aFun]);
    assert_equals(Math[aFun](-0, 0), expected[aFun]);
    assert_equals(Math[aFun](0, 0), 0);
    assert_equals(Math[aFun](-0, -0), -0);
  }, "Should handle negative zero correctly.");
}
