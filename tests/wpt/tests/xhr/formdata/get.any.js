// META: title=FormData: get and getAll

    test(function() {
        assert_equals(create_formdata(['key', 'value1'], ['key', 'value2']).get('key'), "value1");
    }, 'testFormDataGet');
    test(function() {
        assert_equals(create_formdata(['key', 'value1'], ['key', 'value2']).get('nil'), null);
    }, 'testFormDataGetNull1');
    test(function() {
        assert_equals(create_formdata().get('key'), null);
    }, 'testFormDataGetNull2');
    test(function() {
        assert_array_equals(create_formdata(['key', 'value1'], ['key', 'value2']).getAll('key'), ["value1", "value2"]);
    }, 'testFormDataGetAll');
    test(function() {
        assert_array_equals(create_formdata(['key', 'value1'], ['key', 'value2']).getAll('nil'), []);
    }, 'testFormDataGetAllEmpty1');
    test(function() {
        assert_array_equals(create_formdata().getAll('key'), []);
    }, 'testFormDataGetAllEmpty2');

    function create_formdata() {
        var fd = new FormData();
        for (var i = 0; i < arguments.length; i++) {
            fd.append.apply(fd, arguments[i]);
        };
        return fd;
    }
