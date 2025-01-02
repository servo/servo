// META: title=FormData: set

    test(function() {
        assert_equals(create_formdata(['key', 'value1']).get('key'), "value1");
    }, 'testFormDataSet1');
    test(function() {
        assert_equals(create_formdata(['key', 'value2'], ['key', 'value1']).get('key'), "value1");
    }, 'testFormDataSet2');
    test(function() {
        assert_equals(create_formdata(['key', undefined]).get('key'), "undefined");
    }, 'testFormDataSetUndefined1');
    test(function() {
        assert_equals(create_formdata(['key', undefined], ['key', 'value1']).get('key'), "value1");
    }, 'testFormDataSetUndefined2');
    test(function() {
        assert_equals(create_formdata(['key', null]).get('key'), "null");
    }, 'testFormDataSetNull1');
    test(function() {
        assert_equals(create_formdata(['key', null], ['key', 'value1']).get('key'), "value1");
    }, 'testFormDataSetNull2');
    test(function() {
        var fd = new FormData();
        fd.set('key', new Blob([]), 'blank.txt');
        var file = fd.get('key');

        assert_true(file instanceof File);
        assert_equals(file.name, 'blank.txt');
    }, 'testFormDataSetEmptyBlob');

    function create_formdata() {
        var fd = new FormData();
        for (var i = 0; i < arguments.length; i++) {
            fd.set.apply(fd, arguments[i]);
        };
        return fd;
    }
