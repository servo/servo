async_test(function(t) {
    var blob = new Blob(['<a id="a"><b id="b">hey!</b></a>'], {type:"text/plain"});
    var another_blob = new Blob(['<a id="a"><b id="b">hey!</b></a>'], {type:"text/plain"});
    onmessage = t.step_func(function(msg) {
        assert_true(msg.data instanceof Array);
        assert_equals(msg.data.length, 2);
        msg.data.forEach((function(blob, index) {
          assert_true(blob instanceof Blob);
          var cloned_content, original_content;
          var reader = new FileReader();
          reader.addEventListener("loadend", t.step_func(function() {
              original_content = reader.result;
              var reader2 = new FileReader();
              reader2.addEventListener("loadend", t.step_func_done(function() {
                  cloned_content = reader2.result;
                  assert_equals(typeof cloned_content, typeof original_content);
                  assert_equals(cloned_content, original_content);
              }));
              reader2.readAsText(msg.data[index]);
          }));
          reader.readAsText(blob);
        }));
    });
    postMessage([blob, another_blob], '*');
}, "Cloning an array of blobs into the same realm");
