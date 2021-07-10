importScripts("/resources/testharness.js")

async_test(function() {
    var expected = 'Referer: ' +
                   location.href.replace(/[^/]*$/, '') +
                   "workerxhr-origin-referrer.js\n"

    var xhr = new XMLHttpRequest()
    xhr.onreadystatechange = this.step_func(function() {
        if (xhr.readyState == 4) {
            assert_equals(xhr.responseText, expected)
            this.done()
        }
    })
    xhr.open('GET', 'inspect-headers.py?filter_name=referer', true)
    xhr.send()
}, 'Referer header')

async_test(function() {
    var expected = 'Origin: ' +
                   location.protocol +
                   '//' +
                   location.hostname +
                   (location.port === "" ? "" : ":" + location.port) +
                   '\n'

    var xhr = new XMLHttpRequest()
    xhr.onreadystatechange = this.step_func(function() {
        if (xhr.readyState == 4) {
            assert_equals(xhr.responseText, expected)
            this.done()
        }
    })
    var url = location.protocol +
              '//www2.' +
              location.hostname +
              (location.port === "" ? "" : ":" + location.port) +
              location.pathname.replace(/[^/]*$/, '') +
              'inspect-headers.py?filter_name=origin&cors'
    xhr.open('GET', url, true)
    xhr.send()
}, 'Origin header')

async_test(function() {
    // If "origin" / base URL is the origin of this JS file, we can load files
    // from the server it originates from.. and requri.py will be able to tell us
    // what the requested URL was

    var expected = location.href.replace(/[^/]*$/, '') +
                   'requri.py?full'

    var xhr = new XMLHttpRequest()
    xhr.onreadystatechange = this.step_func(function() {
        if (xhr.readyState == 4) {
            assert_equals(xhr.responseText, expected)
            this.done()
        }
    })
    xhr.open('GET', 'requri.py?full', true)
    xhr.send()
}, 'Request URL test')

done()
