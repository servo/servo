// META: title=FormData: has

    test(function() {
        assert_equals(create_formdata(['key', 'value1'], ['key', 'value2']).has('key'), true);
    }, 'testFormDataHas');
    test(function() {
        assert_equals(create_formdata(['key', 'value1'], ['key', 'value2']).has('nil'), false);
    }, 'testFormDataHasEmpty1');
    test(function() {
        assert_equals(create_formdata().has('key'), false);
    }, 'testFormDataHasEmpty2');

    function create_formdata() {
        var fd = new FormData();
        for (var i = 0; i < arguments.length; i++) {
            fd.append.apply(fd, arguments[i]);
        };
        return fd;
    }
