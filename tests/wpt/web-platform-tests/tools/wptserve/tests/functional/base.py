from __future__ import print_function

import base64
import logging
import os
import pytest
import unittest

from six.moves.urllib.parse import urlencode, urlunsplit
from six.moves.urllib.request import Request as BaseRequest
from six.moves.urllib.request import urlopen
from six import binary_type, iteritems, PY3

from hyper import HTTP20Connection, tls
import ssl
from localpaths import repo_root

wptserve = pytest.importorskip("wptserve")

logging.basicConfig()

wptserve.logger.set_logger(logging.getLogger())

here = os.path.split(__file__)[0]
doc_root = os.path.join(here, "docroot")


class Request(BaseRequest):
    def __init__(self, *args, **kwargs):
        BaseRequest.__init__(self, *args, **kwargs)
        self.method = "GET"

    def get_method(self):
        return self.method

    def add_data(self, data):
        if hasattr(data, "items"):
            data = urlencode(data).encode("ascii")

        assert isinstance(data, binary_type)

        if hasattr(BaseRequest, "add_data"):
            BaseRequest.add_data(self, data)
        else:
            self.data = data

        self.add_header("Content-Length", str(len(data)))


class TestUsingServer(unittest.TestCase):
    def setUp(self):
        self.server = wptserve.server.WebTestHttpd(host="localhost",
                                                   port=0,
                                                   use_ssl=False,
                                                   certificate=None,
                                                   doc_root=doc_root)
        self.server.start(False)

    def tearDown(self):
        self.server.stop()

    def abs_url(self, path, query=None):
        return urlunsplit(("http", "%s:%i" % (self.server.host, self.server.port), path, query, None))

    def request(self, path, query=None, method="GET", headers=None, body=None, auth=None):
        req = Request(self.abs_url(path, query))
        req.method = method
        if headers is None:
            headers = {}

        for name, value in iteritems(headers):
            req.add_header(name, value)

        if body is not None:
            req.add_data(body)

        if auth is not None:
            req.add_header("Authorization", b"Basic %s" % base64.b64encode(b"%s:%s" % auth))

        return urlopen(req)

    def assert_multiple_headers(self, resp, name, values):
        if PY3:
            assert resp.info().get_all(name) == values
        else:
            assert resp.info()[name] == ", ".join(values)

@pytest.mark.skipif(not wptserve.utils.http2_compatible(), reason="h2 server only works in python 2.7.15")
class TestUsingH2Server:
    def setup_method(self, test_method):
        self.server = wptserve.server.WebTestHttpd(host="localhost",
                                                   port=0,
                                                   use_ssl=True,
                                                   doc_root=doc_root,
                                                   key_file=os.path.join(repo_root, "tools", "certs", "web-platform.test.key"),
                                                   certificate=os.path.join(repo_root, "tools", "certs", "web-platform.test.pem"),
                                                   handler_cls=wptserve.server.Http2WebTestRequestHandler,
                                                   http2=True)
        self.server.start(False)

        context = tls.init_context()
        context.check_hostname = False
        context.verify_mode = ssl.CERT_NONE
        context.set_alpn_protocols(['h2'])
        self.conn = HTTP20Connection('%s:%i' % (self.server.host, self.server.port), enable_push=True, secure=True, ssl_context=context)
        self.conn.connect()

    def teardown_method(self, test_method):
        self.server.stop()


class TestWrapperHandlerUsingServer(TestUsingServer):
    '''For a wrapper handler, a .js dummy testing file is requried to render
    the html file. This class extends the TestUsingServer and do some some
    extra work: it tries to generate the dummy .js file in setUp and
    remove it in tearDown.'''
    dummy_js_files = {}

    def gen_js_file(self, filename, empty=True, content=b''):
        self.remove_js_file(filename)

        with open(filename, 'wb') as fp:
            if not empty:
                fp.write(content)

    def remove_js_file(self, filename):
        if os.path.exists(filename):
            os.remove(filename)

    def setUp(self):
        super(TestWrapperHandlerUsingServer, self).setUp()

        for filename, content in self.dummy_js_files.items():
            filepath = os.path.join(doc_root, filename)
            if content == '':
                self.gen_js_file(filepath)
            else:
                self.gen_js_file(filepath, False, content)

    def run_wrapper_test(self, req_file, header_data, wrapper_handler):
        route = ('GET', req_file, wrapper_handler())
        self.server.router.register(*route)

        resp = self.request(route[1])
        self.assertEqual(200, resp.getcode())
        self.assertEqual(header_data, resp.info()['Content-Type'])

        with open(os.path.join(doc_root, req_file), 'rb') as fp:
            self.assertEqual(fp.read(), resp.read())

    def tearDown(self):
        super(TestWrapperHandlerUsingServer, self).tearDown()

        for filename, _ in self.dummy_js_files.items():
            filepath = os.path.join(doc_root, filename)
            self.remove_js_file(filepath)
