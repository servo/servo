async_test(function(t) {
    var blob = new Blob(['<a id="a"><b id="b">hey!</b></a>'], {type:"text/plain"});
    onmessage = t.step_func(function(msg) {
        assert_true(msg.data instanceof Blob);
        assert_equals(msg.data.size, blob.size);
        assert_equals(msg.data.type, blob.type);
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
            reader2.readAsText(msg.data);
        }));
        reader.readAsText(blob);
    });
    postMessage(blob, '*');
}, "Cloning a blob into the same realm");
