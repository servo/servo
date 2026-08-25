// META: global=window
// META: timeout=long
// META: variant=?XMLHttpRequest
// META: variant=?TextDecoder
// META: script=resources/encodings.js
// META: script=resources/single-byte-decoder.js

// For TextDecoder tests
var buffer = new ArrayBuffer(255),
    view = new Uint8Array(buffer)
for(var i = 0, l = view.byteLength; i < l; i++) {
  view[i] = i
}

// For XMLHttpRequest and TextDecoder tests
function assert_decode(data, encoding) {
  if(encoding == "ISO-8859-8-I") {
    encoding = "ISO-8859-8"
  }
  for(var i = 0, l = data.length; i < l; i++) {
    var cp = data.charCodeAt(i),
        expectedCp = (i < 0x80) ? i : singleByteIndexes[encoding][i-0x80]
    if(expectedCp == null) {
      expectedCp = 0xFFFD
    }
    assert_equals(cp, expectedCp, encoding + ":" + i)
  }
}

var subsetTest = "";
if (location.search) {
  subsetTest = location.search.substr(1);
}

// Setting up all the tests
for(var i = 0, l = singleByteEncodings.length; i < l; i++) {
  var encoding = singleByteEncodings[i]
  for(var ii = 0, ll = encoding.labels.length; ii < ll; ii++) {
    var label = encoding.labels[ii]

    if (subsetTest == "XMLHttpRequest" || !subsetTest) {
      async_test(function(t) {
        var xhr = new XMLHttpRequest,
            name = encoding.name // need scoped variable
        xhr.open("GET", "resources/single-byte-raw.py?label=" + label)
        xhr.send(null)
        xhr.onload = t.step_func_done(function() { assert_decode(xhr.responseText, name) })
      }, encoding.name + ": " + label + " (XMLHttpRequest)")
    }

    if (subsetTest == "TextDecoder" || !subsetTest) {
      test(function() {
        var d = new TextDecoder(label),
            data = d.decode(view)
        assert_equals(d.encoding, encoding.name.toLowerCase()) // ASCII names only, so safe
        assert_decode(data, encoding.name)
      }, encoding.name + ": " + label + " (TextDecoder)")
    }
  }
}
