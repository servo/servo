function runCharacterSetTests(encodingMap) {
  // Add spaces and mix up case
  Object.keys(encodingMap).forEach(function(name) {
    var lower = encodingMap[name];
    var upper = encodingMap[name].map(function(s) { return s.toUpperCase() });
    var mixed = encodingMap[name].map(function(s) {
      var ret = "";
      for (var i = 0; i < s.length; i += 2) {
        ret += s[i].toUpperCase();
        if (i + 1 < s.length) {
          ret += s[i + 1];
        }
      }
      return ret;
    });
    var spacey = encodingMap[name].map(function(s) {
      return " \t\n\f\r" + s + " \t\n\f\r";
    });
    encodingMap[name] = [];
    for (var i = 0; i < lower.length; i++) {
      encodingMap[name].push(lower[i]);
      /*
      if (lower[i] != upper[i]) {
        encodingMap[name].push(upper[i]);
      }
      if (lower[i] != mixed[i] && upper[i] != mixed[i]) {
        encodingMap[name].push(mixed[i]);
      }
      encodingMap[name].push(spacey[i]);
      */
    }
  });

  Object.keys(encodingMap).forEach(function(name) {
    encodingMap[name].forEach(function(label) {
      var iframe = document.createElement("iframe");
      var t = async_test("Name " + format_value(name) +
                        " has label " + format_value(label) + " (characterSet)");
      var t2 = async_test("Name " + format_value(name) +
                        " has label " + format_value(label) + " (inputEncoding)");
      var t3 = async_test("Name " + format_value(name) +
                        " has label " + format_value(label) + " (charset)");
      iframe.src = "encoding.py?label=" + label;
      iframe.onload = function() {
        t.step(function() {
          assert_equals(iframe.contentDocument.characterSet, name);
        });
        t2.step(function() {
          assert_equals(iframe.contentDocument.inputEncoding, name);
        });
        t3.step(function() {
          assert_equals(iframe.contentDocument.charset, name);
        });
        document.body.removeChild(iframe);
        t.done();
        t2.done();
        t3.done();
      };
      document.body.appendChild(iframe);
    });
  });
}