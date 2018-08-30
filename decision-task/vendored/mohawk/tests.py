import sys
from unittest import TestCase
from base64 import b64decode, urlsafe_b64encode

import mock
from nose.tools import eq_, raises
import six

from . import Receiver, Sender
from .base import Resource
from .exc import (AlreadyProcessed,
                  BadHeaderValue,
                  CredentialsLookupError,
                  InvalidCredentials,
                  MacMismatch,
                  MisComputedContentHash,
                  MissingAuthorization,
                  TokenExpired,
                  InvalidBewit)
from .util import (parse_authorization_header,
                   utc_now,
                   calculate_ts_mac,
                   validate_credentials)
from .bewit import (get_bewit,
                    check_bewit,
                    strip_bewit,
                    parse_bewit)


class Base(TestCase):

    def setUp(self):
        self.credentials = {
            'id': 'my-hawk-id',
            'key': 'my hAwK sekret',
            'algorithm': 'sha256',
        }

        # This callable might be replaced by tests.
        def seen_nonce(id, nonce, ts):
            return False
        self.seen_nonce = seen_nonce

    def credentials_map(self, id):
        # Pretend this is doing something more interesting like looking up
        # a credentials by ID in a database.
        if self.credentials['id'] != id:
            raise LookupError('No credentialsuration for Hawk ID {id}'
                              .format(id=id))
        return self.credentials


class TestConfig(Base):

    @raises(InvalidCredentials)
    def test_no_id(self):
        c = self.credentials.copy()
        del c['id']
        validate_credentials(c)

    @raises(InvalidCredentials)
    def test_no_key(self):
        c = self.credentials.copy()
        del c['key']
        validate_credentials(c)

    @raises(InvalidCredentials)
    def test_no_algo(self):
        c = self.credentials.copy()
        del c['algorithm']
        validate_credentials(c)

    @raises(InvalidCredentials)
    def test_no_credentials(self):
        validate_credentials(None)

    def test_non_dict_credentials(self):
        class WeirdThing(object):
            def __getitem__(self, key):
                return 'whatever'
        validate_credentials(WeirdThing())


class TestSender(Base):

    def setUp(self):
        super(TestSender, self).setUp()
        self.url = 'http://site.com/foo?bar=1'

    def Sender(self, method='GET', **kw):
        credentials = kw.pop('credentials', self.credentials)
        kw.setdefault('content', '')
        kw.setdefault('content_type', '')
        sender = Sender(credentials, self.url, method, **kw)
        return sender

    def receive(self, request_header, url=None, method='GET', **kw):
        credentials_map = kw.pop('credentials_map', self.credentials_map)
        kw.setdefault('content', '')
        kw.setdefault('content_type', '')
        kw.setdefault('seen_nonce', self.seen_nonce)
        return Receiver(credentials_map, request_header,
                        url or self.url, method, **kw)

    def test_get_ok(self):
        method = 'GET'
        sn = self.Sender(method=method)
        self.receive(sn.request_header, method=method)

    def test_post_ok(self):
        method = 'POST'
        sn = self.Sender(method=method)
        self.receive(sn.request_header, method=method)

    def test_post_content_ok(self):
        method = 'POST'
        content = 'foo=bar&baz=2'
        sn = self.Sender(method=method, content=content)
        self.receive(sn.request_header, method=method, content=content)

    def test_post_content_type_ok(self):
        method = 'POST'
        content = '{"bar": "foobs"}'
        content_type = 'application/json'
        sn = self.Sender(method=method, content=content,
                         content_type=content_type)
        self.receive(sn.request_header, method=method, content=content,
                     content_type=content_type)

    def test_post_content_type_with_trailing_charset(self):
        method = 'POST'
        content = '{"bar": "foobs"}'
        content_type = 'application/json; charset=utf8'
        sn = self.Sender(method=method, content=content,
                         content_type=content_type)
        self.receive(sn.request_header, method=method, content=content,
                     content_type='application/json; charset=other')

    @raises(ValueError)
    def test_missing_payload_details(self):
        self.Sender(method='POST', content=None, content_type=None)

    def test_skip_payload_hashing(self):
        method = 'POST'
        content = '{"bar": "foobs"}'
        content_type = 'application/json'
        sn = self.Sender(method=method, content=None, content_type=None,
                         always_hash_content=False)
        self.receive(sn.request_header, method=method, content=content,
                     content_type=content_type,
                     accept_untrusted_content=True)

    @raises(ValueError)
    def test_cannot_skip_content_only(self):
        self.Sender(method='POST', content=None,
                    content_type='application/json')

    @raises(ValueError)
    def test_cannot_skip_content_type_only(self):
        self.Sender(method='POST', content='{"foo": "bar"}',
                    content_type=None)

    @raises(MacMismatch)
    def test_tamper_with_host(self):
        sn = self.Sender()
        self.receive(sn.request_header, url='http://TAMPERED-WITH.com')

    @raises(MacMismatch)
    def test_tamper_with_method(self):
        sn = self.Sender(method='GET')
        self.receive(sn.request_header, method='POST')

    @raises(MacMismatch)
    def test_tamper_with_path(self):
        sn = self.Sender()
        self.receive(sn.request_header,
                     url='http://site.com/TAMPERED?bar=1')

    @raises(MacMismatch)
    def test_tamper_with_query(self):
        sn = self.Sender()
        self.receive(sn.request_header,
                     url='http://site.com/foo?bar=TAMPERED')

    @raises(MacMismatch)
    def test_tamper_with_scheme(self):
        sn = self.Sender()
        self.receive(sn.request_header, url='https://site.com/foo?bar=1')

    @raises(MacMismatch)
    def test_tamper_with_port(self):
        sn = self.Sender()
        self.receive(sn.request_header,
                     url='http://site.com:8000/foo?bar=1')

    @raises(MisComputedContentHash)
    def test_tamper_with_content(self):
        sn = self.Sender()
        self.receive(sn.request_header, content='stuff=nope')

    def test_non_ascii_content(self):
        content = u'Ivan Kristi\u0107'
        sn = self.Sender(content=content)
        self.receive(sn.request_header, content=content)

    @raises(MacMismatch)
    def test_tamper_with_content_type(self):
        sn = self.Sender(method='POST')
        self.receive(sn.request_header, content_type='application/json')

    @raises(AlreadyProcessed)
    def test_nonce_fail(self):

        def seen_nonce(id, nonce, ts):
            return True

        sn = self.Sender()

        self.receive(sn.request_header, seen_nonce=seen_nonce)

    def test_nonce_ok(self):

        def seen_nonce(id, nonce, ts):
            return False

        sn = self.Sender(seen_nonce=seen_nonce)
        self.receive(sn.request_header)

    @raises(TokenExpired)
    def test_expired_ts(self):
        now = utc_now() - 120
        sn = self.Sender(_timestamp=now)
        self.receive(sn.request_header)

    def test_expired_exception_reports_localtime(self):
        now = utc_now()
        ts = now - 120
        sn = self.Sender(_timestamp=ts)  # force expiry

        exc = None
        with mock.patch('mohawk.base.utc_now') as fake_now:
            fake_now.return_value = now
            try:
                self.receive(sn.request_header)
            except:
                etype, exc, tb = sys.exc_info()

        eq_(type(exc), TokenExpired)
        eq_(exc.localtime_in_seconds, now)

    def test_localtime_offset(self):
        now = utc_now() - 120
        sn = self.Sender(_timestamp=now)
        # Without an offset this will raise an expired exception.
        self.receive(sn.request_header, localtime_offset_in_seconds=-120)

    def test_localtime_skew(self):
        now = utc_now() - 120
        sn = self.Sender(_timestamp=now)
        # Without an offset this will raise an expired exception.
        self.receive(sn.request_header, timestamp_skew_in_seconds=120)

    @raises(MacMismatch)
    def test_hash_tampering(self):
        sn = self.Sender()
        header = sn.request_header.replace('hash="', 'hash="nope')
        self.receive(header)

    @raises(MacMismatch)
    def test_bad_secret(self):
        cfg = {
            'id': 'my-hawk-id',
            'key': 'INCORRECT; YOU FAIL',
            'algorithm': 'sha256',
        }
        sn = self.Sender(credentials=cfg)
        self.receive(sn.request_header)

    @raises(MacMismatch)
    def test_unexpected_algorithm(self):
        cr = self.credentials.copy()
        cr['algorithm'] = 'sha512'
        sn = self.Sender(credentials=cr)

        # Validate with mismatched credentials (sha256).
        self.receive(sn.request_header)

    @raises(InvalidCredentials)
    def test_invalid_credentials(self):
        cfg = self.credentials.copy()
        # Create an invalid credentials.
        del cfg['algorithm']

        self.Sender(credentials=cfg)

    @raises(CredentialsLookupError)
    def test_unknown_id(self):
        cr = self.credentials.copy()
        cr['id'] = 'someone-else'
        sn = self.Sender(credentials=cr)

        self.receive(sn.request_header)

    @raises(MacMismatch)
    def test_bad_ext(self):
        sn = self.Sender(ext='my external data')

        header = sn.request_header.replace('my external data', 'TAMPERED')
        self.receive(header)

    def test_ext_with_quotes(self):
        sn = self.Sender(ext='quotes=""')
        self.receive(sn.request_header)
        parsed = parse_authorization_header(sn.request_header)
        eq_(parsed['ext'], 'quotes=""')

    def test_ext_with_new_line(self):
        sn = self.Sender(ext="new line \n in the middle")
        self.receive(sn.request_header)
        parsed = parse_authorization_header(sn.request_header)
        eq_(parsed['ext'], "new line \n in the middle")

    def test_ext_with_equality_sign(self):
        sn = self.Sender(ext="foo=bar&foo2=bar2;foo3=bar3")
        self.receive(sn.request_header)
        parsed = parse_authorization_header(sn.request_header)
        eq_(parsed['ext'], "foo=bar&foo2=bar2;foo3=bar3")

    @raises(BadHeaderValue)
    def test_ext_with_illegal_chars(self):
        self.Sender(ext="something like \t is illegal")

    @raises(BadHeaderValue)
    def test_ext_with_illegal_unicode(self):
        self.Sender(ext=u'Ivan Kristi\u0107')

    @raises(BadHeaderValue)
    def test_ext_with_illegal_utf8(self):
        # This isn't allowed because the escaped byte chars are out of
        # range. It's a little odd but this is what the Node lib does
        # implicitly with its regex.
        self.Sender(ext=u'Ivan Kristi\u0107'.encode('utf8'))

    def test_app_ok(self):
        app = 'custom-app'
        sn = self.Sender(app=app)
        self.receive(sn.request_header)
        parsed = parse_authorization_header(sn.request_header)
        eq_(parsed['app'], app)

    @raises(MacMismatch)
    def test_tampered_app(self):
        app = 'custom-app'
        sn = self.Sender(app=app)
        header = sn.request_header.replace(app, 'TAMPERED-WITH')
        self.receive(header)

    def test_dlg_ok(self):
        dlg = 'custom-dlg'
        sn = self.Sender(dlg=dlg)
        self.receive(sn.request_header)
        parsed = parse_authorization_header(sn.request_header)
        eq_(parsed['dlg'], dlg)

    @raises(MacMismatch)
    def test_tampered_dlg(self):
        dlg = 'custom-dlg'
        sn = self.Sender(dlg=dlg, app='some-app')
        header = sn.request_header.replace(dlg, 'TAMPERED-WITH')
        self.receive(header)


class TestReceiver(Base):

    def setUp(self):
        super(TestReceiver, self).setUp()
        self.url = 'http://site.com/'
        self.sender = None
        self.receiver = None

    def receive(self, method='GET', **kw):
        url = kw.pop('url', self.url)
        sender = kw.pop('sender', None)
        sender_kw = kw.pop('sender_kw', {})
        sender_kw.setdefault('content', '')
        sender_kw.setdefault('content_type', '')
        sender_url = kw.pop('sender_url', url)

        credentials_map = kw.pop('credentials_map',
                                 lambda id: self.credentials)

        if sender:
            self.sender = sender
        else:
            self.sender = Sender(self.credentials, sender_url, method,
                                 **sender_kw)

        kw.setdefault('content', '')
        kw.setdefault('content_type', '')
        self.receiver = Receiver(credentials_map,
                                 self.sender.request_header, url, method,
                                 **kw)

    def respond(self, **kw):
        accept_kw = kw.pop('accept_kw', {})
        accept_kw.setdefault('content', '')
        accept_kw.setdefault('content_type', '')
        receiver = kw.pop('receiver', self.receiver)

        kw.setdefault('content', '')
        kw.setdefault('content_type', '')
        receiver.respond(**kw)
        self.sender.accept_response(receiver.response_header, **accept_kw)

        return receiver.response_header

    @raises(InvalidCredentials)
    def test_invalid_credentials_lookup(self):
        # Return invalid credentials.
        self.receive(credentials_map=lambda *a: {})

    def test_get_ok(self):
        method = 'GET'
        self.receive(method=method)
        self.respond()

    def test_post_ok(self):
        method = 'POST'
        self.receive(method=method)
        self.respond()

    @raises(MisComputedContentHash)
    def test_respond_with_wrong_content(self):
        self.receive()
        self.respond(content='real content',
                     accept_kw=dict(content='TAMPERED WITH'))

    @raises(MisComputedContentHash)
    def test_respond_with_wrong_content_type(self):
        self.receive()
        self.respond(content_type='text/html',
                     accept_kw=dict(content_type='application/json'))

    @raises(MissingAuthorization)
    def test_missing_authorization(self):
        Receiver(lambda id: self.credentials, None, '/', 'GET')

    @raises(MacMismatch)
    def test_respond_with_wrong_url(self):
        self.receive(url='http://fakesite.com')
        wrong_receiver = self.receiver

        self.receive(url='http://realsite.com')

        self.respond(receiver=wrong_receiver)

    @raises(MacMismatch)
    def test_respond_with_wrong_method(self):
        self.receive(method='GET')
        wrong_receiver = self.receiver

        self.receive(method='POST')

        self.respond(receiver=wrong_receiver)

    @raises(MacMismatch)
    def test_respond_with_wrong_nonce(self):
        self.receive(sender_kw=dict(nonce='another-nonce'))
        wrong_receiver = self.receiver

        self.receive()

        # The nonce must match the one sent in the original request.
        self.respond(receiver=wrong_receiver)

    def test_respond_with_unhashed_content(self):
        self.receive()

        self.respond(always_hash_content=False, content=None,
                     content_type=None,
                     accept_kw=dict(accept_untrusted_content=True))

    @raises(TokenExpired)
    def test_respond_with_expired_ts(self):
        self.receive()
        hdr = self.receiver.respond(content='', content_type='')

        with mock.patch('mohawk.base.utc_now') as fn:
            fn.return_value = 0  # force an expiry
            try:
                self.sender.accept_response(hdr, content='', content_type='')
            except TokenExpired:
                etype, exc, tb = sys.exc_info()
                hdr = parse_authorization_header(exc.www_authenticate)
                calculated = calculate_ts_mac(fn(), self.credentials)
                if isinstance(calculated, six.binary_type):
                    calculated = calculated.decode('ascii')
                eq_(hdr['tsm'], calculated)
                raise

    def test_respond_with_bad_ts_skew_ok(self):
        now = utc_now() - 120

        self.receive()
        hdr = self.receiver.respond(content='', content_type='')

        with mock.patch('mohawk.base.utc_now') as fn:
            fn.return_value = now

            # Without an offset this will raise an expired exception.
            self.sender.accept_response(hdr, content='', content_type='',
                                        timestamp_skew_in_seconds=120)

    def test_respond_with_ext(self):
        self.receive()

        ext = 'custom-ext'
        self.respond(ext=ext)
        header = parse_authorization_header(self.receiver.response_header)
        eq_(header['ext'], ext)

    @raises(MacMismatch)
    def test_respond_with_wrong_app(self):
        self.receive(sender_kw=dict(app='TAMPERED-WITH', dlg='delegation'))
        self.receiver.respond(content='', content_type='')
        wrong_receiver = self.receiver

        self.receive(sender_kw=dict(app='real-app', dlg='delegation'))

        self.sender.accept_response(wrong_receiver.response_header,
                                    content='', content_type='')

    @raises(MacMismatch)
    def test_respond_with_wrong_dlg(self):
        self.receive(sender_kw=dict(app='app', dlg='TAMPERED-WITH'))
        self.receiver.respond(content='', content_type='')
        wrong_receiver = self.receiver

        self.receive(sender_kw=dict(app='app', dlg='real-dlg'))

        self.sender.accept_response(wrong_receiver.response_header,
                                    content='', content_type='')

    @raises(MacMismatch)
    def test_receive_wrong_method(self):
        self.receive(method='GET')
        wrong_sender = self.sender
        self.receive(method='POST', sender=wrong_sender)

    @raises(MacMismatch)
    def test_receive_wrong_url(self):
        self.receive(url='http://fakesite.com/')
        wrong_sender = self.sender
        self.receive(url='http://realsite.com/', sender=wrong_sender)

    @raises(MisComputedContentHash)
    def test_receive_wrong_content(self):
        self.receive(sender_kw=dict(content='real request'),
                     content='real request')
        wrong_sender = self.sender
        self.receive(content='TAMPERED WITH', sender=wrong_sender)

    @raises(MisComputedContentHash)
    def test_unexpected_unhashed_content(self):
        self.receive(sender_kw=dict(content=None, content_type=None,
                                    always_hash_content=False))

    @raises(ValueError)
    def test_cannot_receive_empty_content_only(self):
        content_type = 'text/plain'
        self.receive(sender_kw=dict(content='<content>',
                                    content_type=content_type),
                     content=None, content_type=content_type)

    @raises(ValueError)
    def test_cannot_receive_empty_content_type_only(self):
        content = '<content>'
        self.receive(sender_kw=dict(content=content,
                                    content_type='text/plain'),
                     content=content, content_type=None)

    @raises(MisComputedContentHash)
    def test_receive_wrong_content_type(self):
        self.receive(sender_kw=dict(content_type='text/html'),
                     content_type='text/html')
        wrong_sender = self.sender

        self.receive(content_type='application/json',
                     sender=wrong_sender)


class TestSendAndReceive(Base):

    def test(self):
        credentials = {
            'id': 'some-id',
            'key': 'some secret',
            'algorithm': 'sha256'
        }

        url = 'https://my-site.com/'
        method = 'POST'

        # The client sends a request with a Hawk header.
        content = 'foo=bar&baz=nooz'
        content_type = 'application/x-www-form-urlencoded'

        sender = Sender(credentials,
                        url, method,
                        content=content,
                        content_type=content_type)

        # The server receives a request and authorizes access.
        receiver = Receiver(lambda id: credentials,
                            sender.request_header,
                            url, method,
                            content=content,
                            content_type=content_type)

        # The server responds with a similar Hawk header.
        content = 'we are friends'
        content_type = 'text/plain'
        receiver.respond(content=content,
                         content_type=content_type)

        # The client receives a response and authorizes access.
        sender.accept_response(receiver.response_header,
                               content=content,
                               content_type=content_type)


class TestBewit(Base):

    # Test cases copied from
    # https://github.com/hueniverse/hawk/blob/492632da51ecedd5f59ce96f081860ad24ce6532/test/uri.js

    def setUp(self):
        self.credentials = {
            'id': '123456',
            'key': '2983d45yun89q',
            'algorithm': 'sha256',
        }

    def make_credential_lookup(self, credentials_map):
        # Helper function to make a lookup function given a dictionary of
        # credentials
        def lookup(client_id):
            # Will raise a KeyError if missing; which is a subclass of
            # LookupError
            return credentials_map[client_id]
        return lookup

    def test_bewit(self):
        res = Resource(url='https://example.com/somewhere/over/the/rainbow',
                       method='GET', credentials=self.credentials,
                       timestamp=1356420407 + 300,
                       nonce='',
                       )
        bewit = get_bewit(res)

        expected = '123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        eq_(b64decode(bewit).decode('ascii'), expected)

    def test_bewit_with_binary_id(self):
        # Check for exceptions in get_bewit call with binary id
        binary_credentials = self.credentials.copy()
        binary_credentials['id'] = binary_credentials['id'].encode('ascii')
        res = Resource(url='https://example.com/somewhere/over/the/rainbow',
                       method='GET', credentials=binary_credentials,
                       timestamp=1356420407 + 300,
                       nonce='',
                       )
        get_bewit(res)

    def test_bewit_with_ext(self):
        res = Resource(url='https://example.com/somewhere/over/the/rainbow',
                       method='GET', credentials=self.credentials,
                       timestamp=1356420407 + 300,
                       nonce='',
                       ext='xandyandz'
                       )
        bewit = get_bewit(res)

        expected = '123456\\1356420707\\kscxwNR2tJpP1T1zDLNPbB5UiKIU9tOSJXTUdG7X9h8=\\xandyandz'
        eq_(b64decode(bewit).decode('ascii'), expected)

    def test_bewit_with_ext_and_backslashes(self):
        credentials = self.credentials
        credentials['id'] = '123\\456'
        res = Resource(url='https://example.com/somewhere/over/the/rainbow',
                       method='GET', credentials=self.credentials,
                       timestamp=1356420407 + 300,
                       nonce='',
                       ext='xand\\yandz'
                       )
        bewit = get_bewit(res)

        expected = '123456\\1356420707\\b82LLIxG5UDkaChLU953mC+SMrbniV1sb8KiZi9cSsc=\\xand\\yandz'
        eq_(b64decode(bewit).decode('ascii'), expected)

    def test_bewit_with_port(self):
        res = Resource(url='https://example.com:8080/somewhere/over/the/rainbow',
                       method='GET', credentials=self.credentials,
                       timestamp=1356420407 + 300, nonce='', ext='xandyandz')
        bewit = get_bewit(res)

        expected = '123456\\1356420707\\hZbJ3P2cKEo4ky0C8jkZAkRyCZueg4WSNbxV7vq3xHU=\\xandyandz'
        eq_(b64decode(bewit).decode('ascii'), expected)

    @raises(ValueError)
    def test_bewit_with_nonce(self):
        res = Resource(url='https://example.com/somewhere/over/the/rainbow',
                       method='GET', credentials=self.credentials,
                       timestamp=1356420407 + 300,
                       nonce='n1')
        get_bewit(res)

    @raises(ValueError)
    def test_bewit_invalid_method(self):
        res = Resource(url='https://example.com:8080/somewhere/over/the/rainbow',
                       method='POST', credentials=self.credentials,
                       timestamp=1356420407 + 300, nonce='')
        get_bewit(res)

    def test_strip_bewit(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)

        raw_bewit, stripped_url = strip_bewit(url)
        self.assertEquals(raw_bewit, bewit)
        self.assertEquals(stripped_url, "https://example.com/somewhere/over/the/rainbow")

    @raises(InvalidBewit)
    def test_strip_url_without_bewit(self):
        url = "https://example.com/somewhere/over/the/rainbow"
        strip_bewit(url)

    def test_parse_bewit(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        bewit = parse_bewit(bewit)
        self.assertEquals(bewit.id, '123456')
        self.assertEquals(bewit.expiration, '1356420707')
        self.assertEquals(bewit.mac, 'IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=')
        self.assertEquals(bewit.ext, '')

    def test_parse_bewit_with_ext(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\xandyandz'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        bewit = parse_bewit(bewit)
        self.assertEquals(bewit.id, '123456')
        self.assertEquals(bewit.expiration, '1356420707')
        self.assertEquals(bewit.mac, 'IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=')
        self.assertEquals(bewit.ext, 'xandyandz')

    def test_parse_bewit_with_ext_and_backslashes(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\xand\\yandz'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        bewit = parse_bewit(bewit)
        self.assertEquals(bewit.id, '123456')
        self.assertEquals(bewit.expiration, '1356420707')
        self.assertEquals(bewit.mac, 'IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=')
        self.assertEquals(bewit.ext, 'xand\\yandz')

    @raises(InvalidBewit)
    def test_parse_invalid_bewit_with_only_one_part(self):
        bewit = b'12345'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        bewit = parse_bewit(bewit)

    @raises(InvalidBewit)
    def test_parse_invalid_bewit_with_only_two_parts(self):
        bewit = b'1\\2'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        bewit = parse_bewit(bewit)

    def test_validate_bewit(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)
        credential_lookup = self.make_credential_lookup({
            self.credentials['id']: self.credentials,
        })
        self.assertTrue(check_bewit(url, credential_lookup=credential_lookup, now=1356420407 + 10))

    def test_validate_bewit_with_ext(self):
        bewit = b'123456\\1356420707\\kscxwNR2tJpP1T1zDLNPbB5UiKIU9tOSJXTUdG7X9h8=\\xandyandz'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)
        credential_lookup = self.make_credential_lookup({
            self.credentials['id']: self.credentials,
        })
        self.assertTrue(check_bewit(url, credential_lookup=credential_lookup, now=1356420407 + 10))

    def test_validate_bewit_with_ext_and_backslashes(self):
        bewit = b'123456\\1356420707\\b82LLIxG5UDkaChLU953mC+SMrbniV1sb8KiZi9cSsc=\\xand\\yandz'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)
        credential_lookup = self.make_credential_lookup({
            self.credentials['id']: self.credentials,
        })
        self.assertTrue(check_bewit(url, credential_lookup=credential_lookup, now=1356420407 + 10))

    @raises(TokenExpired)
    def test_validate_expired_bewit(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)
        credential_lookup = self.make_credential_lookup({
            self.credentials['id']: self.credentials,
        })
        check_bewit(url, credential_lookup=credential_lookup, now=1356420407 + 1000)

    @raises(CredentialsLookupError)
    def test_validate_bewit_with_unknown_credentials(self):
        bewit = b'123456\\1356420707\\IGYmLgIqLrCe8CxvKPs4JlWIA+UjWJJouwgARiVhCAg=\\'
        bewit = urlsafe_b64encode(bewit).decode('ascii')
        url = "https://example.com/somewhere/over/the/rainbow?bewit={bewit}".format(bewit=bewit)
        credential_lookup = self.make_credential_lookup({
            'other_id': self.credentials,
        })
        check_bewit(url, credential_lookup=credential_lookup, now=1356420407 + 10)
