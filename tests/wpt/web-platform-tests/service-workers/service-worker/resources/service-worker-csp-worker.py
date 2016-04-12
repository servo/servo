bodyDefault = '''
importScripts('worker-testharness.js');
importScripts('test-helpers.sub.js');
importScripts('../resources/get-host-info.sub.js');

var host_info = get_host_info();

test(function() {
    var import_script_failed = false;
    try {
      importScripts(host_info.HTTPS_REMOTE_ORIGIN +
        base_path() + 'empty.js');
    } catch(e) {
      import_script_failed = true;
    }
    assert_true(import_script_failed,
                'Importing the other origins script should fail.');
  }, 'importScripts test for default-src');

async_test(function(t) {
    fetch(host_info.HTTPS_REMOTE_ORIGIN +
          base_path() + 'fetch-access-control.py?ACAOrigin=*',
          {mode: 'cors'})
      .then(function(response){
          assert_unreached('fetch should fail.');
        }, function(){
          t.done();
        })
      .catch(unreached_rejection(t));
  }, 'Fetch test for default-src');

async_test(function(t) {
    var REDIRECT_URL = host_info.HTTPS_ORIGIN +
      base_path() + 'redirect.py?Redirect=';
    var OTHER_BASE_URL = host_info.HTTPS_REMOTE_ORIGIN +
      base_path() + 'fetch-access-control.py?'
    fetch(REDIRECT_URL + encodeURIComponent(OTHER_BASE_URL + 'ACAOrigin=*'),
          {mode: 'cors'})
      .then(function(response){
          assert_unreached('Redirected fetch should fail.');
        }, function(){
          t.done();
        })
      .catch(unreached_rejection(t));
  }, 'Redirected fetch test for default-src');'''

bodyScript = '''
importScripts('worker-testharness.js');
importScripts('test-helpers.sub.js');
importScripts('../resources/get-host-info.sub.js');

var host_info = get_host_info();

test(function() {
    var import_script_failed = false;
    try {
      importScripts(host_info.HTTPS_REMOTE_ORIGIN +
        base_path() + 'empty.js');
    } catch(e) {
      import_script_failed = true;
    }
    assert_true(import_script_failed,
                'Importing the other origins script should fail.');
  }, 'importScripts test for script-src');

async_test(function(t) {
    fetch(host_info.HTTPS_REMOTE_ORIGIN +
          base_path() + 'fetch-access-control.py?ACAOrigin=*',
          {mode: 'cors'})
      .then(function(response){
          t.done();
        }, function(){
          assert_unreached('fetch should not fail.');
        })
      .catch(unreached_rejection(t));
  }, 'Fetch test for script-src');

async_test(function(t) {
    var REDIRECT_URL = host_info.HTTPS_ORIGIN +
      base_path() + 'redirect.py?Redirect=';
    var OTHER_BASE_URL = host_info.HTTPS_REMOTE_ORIGIN +
      base_path() + 'fetch-access-control.py?'
    fetch(REDIRECT_URL + encodeURIComponent(OTHER_BASE_URL + 'ACAOrigin=*'),
          {mode: 'cors'})
      .then(function(response){
          t.done();
        }, function(){
          assert_unreached('Redirected fetch should not fail.');
        })
      .catch(unreached_rejection(t));
  }, 'Redirected fetch test for script-src');'''

bodyConnect = '''
importScripts('worker-testharness.js');
importScripts('test-helpers.sub.js');
importScripts('../resources/get-host-info.sub.js');

var host_info = get_host_info();

test(function() {
    var import_script_failed = false;
    try {
      importScripts(host_info.HTTPS_REMOTE_ORIGIN +
        base_path() + 'empty.js');
    } catch(e) {
      import_script_failed = true;
    }
    assert_false(import_script_failed,
                 'Importing the other origins script should not fail.');
  }, 'importScripts test for connect-src');

async_test(function(t) {
    fetch(host_info.HTTPS_REMOTE_ORIGIN +
          base_path() + 'fetch-access-control.py?ACAOrigin=*',
          {mode: 'cors'})
      .then(function(response){
          assert_unreached('fetch should fail.');
        }, function(){
          t.done();
        })
      .catch(unreached_rejection(t));
  }, 'Fetch test for connect-src');

async_test(function(t) {
    var REDIRECT_URL = host_info.HTTPS_ORIGIN +
      base_path() + 'redirect.py?Redirect=';
    var OTHER_BASE_URL = host_info.HTTPS_REMOTE_ORIGIN +
      base_path() + 'fetch-access-control.py?'
    fetch(REDIRECT_URL + encodeURIComponent(OTHER_BASE_URL + 'ACAOrigin=*'),
          {mode: 'cors'})
      .then(function(response){
          assert_unreached('Redirected fetch should fail.');
        }, function(){
          t.done();
        })
      .catch(unreached_rejection(t));
  }, 'Redirected fetch test for connect-src');'''

def main(request, response):
    headers = []
    headers.append(('Content-Type', 'application/javascript'))
    directive = request.GET['directive']
    body = 'ERROR: Unknown directive'
    if directive == 'default':
        headers.append(('Content-Security-Policy', "default-src 'self'"))
        body = bodyDefault
    elif directive == 'script':
        headers.append(('Content-Security-Policy', "script-src 'self'"))
        body = bodyScript
    elif directive == 'connect':
        headers.append(('Content-Security-Policy', "connect-src 'self'"))
        body = bodyConnect
    return headers, body
