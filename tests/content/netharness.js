function assert_requests_made(url, n) {
    var x = new XMLHttpRequest();
    x.open('GET', 'stats?' + url, false);
    x.send();
    is(parseInt(x.responseText), n, '# of requests for ' + url + ' should be ' + n);
}

function reset_stats() {
    var x = new XMLHttpRequest();
    x.open('POST', 'reset', false);
    x.send();
    is(x.status, 200, 'resetting stats should succeed');    
}

function fetch(url) {
    var x = new XMLHttpRequest();
    x.open('GET', url, false);
    x.send();
    is(x.status, 200, 'fetching ' + url + ' should succeed ');    
}
