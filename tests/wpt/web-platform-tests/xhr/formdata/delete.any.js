// META: title=FormData: delete

    test(function() {
        var fd = create_formdata(['key', 'value1'], ['key', 'value2']);
        fd.delete('key');
        assert_equals(fd.get('key'), null);
    }, 'testFormDataDelete');
    test(function() {
        var fd = create_formdata(['key', 'value1'], ['key', 'value2']);
        fd.delete('nil');
        assert_equals(fd.get('key'), 'value1');
    }, 'testFormDataDeleteNonExistentKey');
    test(function() {
        var fd = create_formdata(['key1', 'value1'], ['key2', 'value2']);
        fd.delete('key1');
        assert_equals(fd.get('key1'), null);
        assert_equals(fd.get('key2'), 'value2');
    }, 'testFormDataDeleteOtherKey');

    function create_formdata() {
        var fd = new FormData();
        for (var i = 0; i < arguments.length; i++) {
            fd.append.apply(fd, arguments[i]);
        };
        return fd;
    }
