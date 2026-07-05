// META: timeout=long
// META: script=resources/encodings.js
// META: script=resources/single-byte-decoder.js

for (var i = 0, l = singleByteEncodings.length; i < l; i++) {
  var encoding = singleByteEncodings[i]
  for (var ii = 0, ll = encoding.labels.length; ii < ll; ii++) {
    var label = encoding.labels[ii]
    async_test(function(t) {
      var frame = document.createElement("iframe"),
          name = encoding.name;
      frame.src = "resources/text-plain-charset.py?label=" + label
      frame.onload = t.step_func_done(function() {
        assert_equals(frame.contentDocument.characterSet, name)
        assert_equals(frame.contentDocument.inputEncoding, name)
      })
      t.add_cleanup(function() { document.body.removeChild(frame) })
      document.body.appendChild(frame)
    }, encoding.name + ": " + label + " (document.characterSet and document.inputEncoding)")
  }
}
