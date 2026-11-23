test(function() {
  var invalid = [
        ["A", "?>"],
        ["", "x"],
        // Invalid start or continuation characters:
        ["-A", "x"],
        [".A", "x"],
        ["0A", "x"],
        [" A", "x"],
        ["\tA", "x"],
        ["A B", "x"],
        ["A\tB", "x"],
        ["\u00AA", "x"],
        ["A\u00AA", "x"],
        ["\u00BA", "x"],
        ["A\u00BA", "x"],
        ["\u00B7A", "x"],
        ["\u00D7A", "x"],
        ["A\u00D7", "x"],
        ["\u00F7A", "x"],
        ["A\u00F7", "x"],
        ["\u037E", "x"],
        ["\u2000", "x"],
        ["\uD800", "x"],
        ["\uFFFE", "x"],
        ["\uFFFF", "x"],
        ["\u{F0000}", "x"],
        ["\u{10FFFF}", "x"],
        ["\\A", "x"],
        ["\f", "x"],
        [0, "x"],
        ["0", "x"]
      ],
      valid = [
        ["xml:fail", "x"],
        [":", "x"],
        ["_0", "x"],
        ["A-B", "x"],
        ["A.B", "x"],
        ["A\u00B7A", "x"],
        ["A\u0300", "x"],
        ["A\u203F", "x"],
        ["a0", "x"],
        // Characters added to NameStartChar in XML 1.0 Fifth Edition:
        ["\u02B3A", "x"],
        ["\u0591A.B", "x"],
        ["\u093FA", "x"],
        ["\u20E3A", "x"],
        ["A\u2166", "x"],
        ["\uF900", "x"],
        ["\uFFFD", "x"],
        ["\u{10000}", "x"],
        ["\u{1F600}", "x"],
        ["\u{EFFFF}", "x"]
      ]

  for (var i = 0, il = invalid.length; i < il; i++) {
    test(function() {
      assert_throws_dom("INVALID_CHARACTER_ERR", function() {
        document.createProcessingInstruction(invalid[i][0], invalid[i][1])
      })
    }, "Should throw an INVALID_CHARACTER_ERR for target " +
       format_value(invalid[i][0]) + " and data " +
       format_value(invalid[i][1]) + ".")
  }
  for (var i = 0, il = valid.length; i < il; ++i) {
    test(function() {
      var pi = document.createProcessingInstruction(valid[i][0], valid[i][1]);
      assert_equals(pi.target, valid[i][0]);
      assert_equals(pi.data, valid[i][1]);
      assert_equals(pi.ownerDocument, document);
      assert_true(pi instanceof ProcessingInstruction);
      assert_true(pi instanceof Node);
    }, "Should get a ProcessingInstruction for target " +
      format_value(valid[i][0]) + " and data " +
      format_value(valid[i][1]) + ".")
  }
})
