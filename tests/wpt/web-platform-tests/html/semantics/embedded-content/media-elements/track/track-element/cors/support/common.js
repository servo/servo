setup(function(){
    window.id = token();
    var p = document.createElement('p');
    p.innerHTML = 'Test id: <samp>'+id+'</samp>';
    document.body.appendChild(p);
    window.actual = {event:null, requests:[]};
    window.errors = [];
    window.origin = location.protocol+'//'+location.host;
    window.escapedOrigin = encodeURIComponent(origin);
    window.sameOriginURL = "http://{{domains[]}}:{{ports[http][0]}}" + location.pathname.replace(/\/[^\/]+$/, '/');
    window.otherOriginURL = "http://{{domains[www1]}}:{{ports[http][0]}}" + location.pathname.replace(/\/[^\/]+$/, '/');
}, {timeout:10000, explicit_done:true});

onload = function() {
    (async_test(document.title, {timeout:10000})).step(function() {
        // fail early if track isn't supported
        assert_true('HTMLTrackElement' in window, 'track not supported');
        window.corsMode = document.title.match(/^track CORS: (No CORS|Anonymous|Use Credentials)/)[1];
        var requests_tmp = document.title.substr(('track CORS: '+corsMode+', ').length).split(/, redirects to /g);
        window.requests = [];
        requests_tmp.forEach(function(r) {
            var parts = r.split(', ');
            requests.push({sameOrigin:parts[0] == 'same-origin', withHeaders:parts[1] == 'with headers'});
        });
        if (document.title.indexOf('not same-origin') > -1) {
            window.hasCrossDomainCookie = true;
            this.step(setCrossDomainCookie);
        } else {
            window.hasCrossDomainCookie = false;
            this.step(loadTrack);
        }
    });
    done();
};

function setCrossDomainCookie() {
    var iframe = document.createElement('iframe');
    iframe.onload = this.step_func(loadTrack);
    iframe.src = otherOriginURL + 'support/set-cookie.html#'+id;
    document.body.appendChild(iframe);
}

function loadTrack() {
    var video = document.createElement('video');
    window.track = document.createElement('track');
    if (corsMode == 'Anonymous')
        video.setAttribute('crossorigin', 'anonymous');
    else if (corsMode == 'Use Credentials')
        video.setAttribute('crossorigin', 'use-credentials');
    // else No CORS, omit the crossorigin attribute
    video.appendChild(track);
    document.body.appendChild(video);
    track.track.mode = 'showing';
    document.cookie = id+'=yes;path=/;max-age=10';
    var url = '';
    var r;
    while (r = requests.pop()) {
        url = (r.sameOrigin ? sameOriginURL : otherOriginURL) +
              'support/cors-tester.py?id=' + id +
              (r.withHeaders ? '&origin=' + escapedOrigin : '') +
              (url === '' ? '' : '&redirect=' + encodeURIComponent(url));
    }
    track.src = url;
    track.onerror = track.onload = this.step_func(function(e) {
        actual.event = e.type;
        var xhr = new XMLHttpRequest();
        xhr.open('GET', 'support/cors-tester.py?read=true&id=' + id, true);
        xhr.onload = this.step_func(function() {
            if (xhr.status == 200) {
                var lines = xhr.responseText.split('\n');
                lines.forEach(function(line) {
                    var chunks = line.split(' | ');
                    var current = {};
                    actual.requests.push(current);
                    chunks.forEach(function(chunk) {
                        var nameval = chunk.split(' = ');
                        var name = nameval[0];
                        var value = nameval[1];
                        current[name] = value;
                    });
                });
            } else if (xhr.status == 404) {
                //No stash was found
            } else {
                errors.push('got unexpected xhr status: '+xhr.status);
            }
            this.step(removeCookies);
        });
        xhr.onerror = this.step_func(function() {
            errors.push('got xhr error');
            this.step(removeCookies);
        });
        xhr.send();
    });
}

function removeCookies() {
    document.cookie = id+'=;path=/;max-age=0';
    var nextStep = checkData;
    if (hasCrossDomainCookie) {
        var iframe = document.createElement('iframe');
        iframe.onload = this.step_func(nextStep);
        iframe.src = otherOriginURL + 'support/cors-tester.py?delete-cookie&id=' + id;
        document.body.appendChild(iframe);
    } else {
        this.step(nextStep);
    }
}

function removeLog() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', 'support/cors-tester.py?cleanup&id='+id, true);
    xhr.onload = this.step_func(function() {
        assert_equals(xhr.responseText, 'OK', 'failed to clean up log: '+id);
        this.step(checkData);
    });
    xhr.onerror = this.step_func(function() {
        assert_unreached('failed to clean up log: '+id);
    });
    xhr.send();
}

function checkData() {
    assert_equals(errors.length, 0, errors);
    try {
        if (actual.event == 'load' && expected.event == 'error')
            assert_unreached('Security problem: got load event but expected error event');
        assert_object_equals(actual, expected);
    } catch(ex) {
        var style = document.createElement('style');
        style.textContent = '.json-diffs td { vertical-align:top } .json-diffs pre { margin:0 }';
        document.head.appendChild(style);
        var table = document.createElement('table');
        table.border = "";
        table.className = 'json-diffs';
        table.innerHTML = '<tr><th>Actual<th>Expected<tr><td><pre></pre><td><pre></pre>';
        table.getElementsByTagName('pre')[0].textContent = JSON.stringify(actual, null, 2);
        table.getElementsByTagName('pre')[1].textContent = JSON.stringify(expected, null, 2);
        document.body.insertBefore(table, document.getElementById('log'));
        throw ex;
    }
    assert_equals(track.track.cues.length, expected.event == 'load' ? 1 : 0, 'track.track.cues.length');
    this.done();
}
