import binascii
import datetime
import ssl
from unittest import TestCase
from unittest.mock import patch

from cryptography.exceptions import UnsupportedAlgorithm
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec

from aioquic import tls
from aioquic.buffer import Buffer, BufferReadError
from aioquic.quic.configuration import QuicConfiguration
from aioquic.tls import (
    Certificate,
    CertificateVerify,
    ClientHello,
    Context,
    EncryptedExtensions,
    Finished,
    NewSessionTicket,
    ServerHello,
    State,
    load_pem_x509_certificates,
    pull_block,
    pull_certificate,
    pull_certificate_verify,
    pull_client_hello,
    pull_encrypted_extensions,
    pull_finished,
    pull_new_session_ticket,
    pull_server_hello,
    push_certificate,
    push_certificate_verify,
    push_client_hello,
    push_encrypted_extensions,
    push_finished,
    push_new_session_ticket,
    push_server_hello,
    verify_certificate,
)

from .utils import (
    SERVER_CACERTFILE,
    SERVER_CERTFILE,
    SERVER_KEYFILE,
    generate_ec_certificate,
    load,
)

CERTIFICATE_DATA = load("tls_certificate.bin")[11:-2]
CERTIFICATE_VERIFY_SIGNATURE = load("tls_certificate_verify.bin")[-384:]

CLIENT_QUIC_TRANSPORT_PARAMETERS = binascii.unhexlify(
    b"ff0000110031000500048010000000060004801000000007000480100000000"
    b"4000481000000000100024258000800024064000a00010a"
)

SERVER_QUIC_TRANSPORT_PARAMETERS = binascii.unhexlify(
    b"ff00001104ff000011004500050004801000000006000480100000000700048"
    b"010000000040004810000000001000242580002001000000000000000000000"
    b"000000000000000800024064000a00010a"
)

SERVER_QUIC_TRANSPORT_PARAMETERS_2 = binascii.unhexlify(
    b"0057000600048000ffff000500048000ffff00020010c5ac410fbdd4fe6e2c1"
    b"42279f231e8e0000a000103000400048005fffa000b000119000100026710ff"
    b"42000c5c067f27e39321c63e28e7c90003000247e40008000106"
)

SERVER_QUIC_TRANSPORT_PARAMETERS_3 = binascii.unhexlify(
    b"0054000200100dcb50a442513295b4679baf04cb5effff8a0009c8afe72a6397"
    b"255407000600048000ffff0008000106000400048005fffa000500048000ffff"
    b"0003000247e4000a000103000100026710000b000119"
)


class BufferTest(TestCase):
    def test_pull_block_truncated(self):
        buf = Buffer(capacity=0)
        with self.assertRaises(BufferReadError):
            with pull_block(buf, 1):
                pass


def create_buffers():
    return {
        tls.Epoch.INITIAL: Buffer(capacity=4096),
        tls.Epoch.HANDSHAKE: Buffer(capacity=4096),
        tls.Epoch.ONE_RTT: Buffer(capacity=4096),
    }


def merge_buffers(buffers):
    return b"".join(x.data for x in buffers.values())


def reset_buffers(buffers):
    for k in buffers.keys():
        buffers[k].seek(0)


class ContextTest(TestCase):
    def create_client(
        self, alpn_protocols=None, cadata=None, cafile=SERVER_CACERTFILE, **kwargs
    ):
        client = Context(
            alpn_protocols=alpn_protocols,
            cadata=cadata,
            cafile=cafile,
            is_client=True,
            **kwargs
        )
        client.handshake_extensions = [
            (
                tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                CLIENT_QUIC_TRANSPORT_PARAMETERS,
            )
        ]
        self.assertEqual(client.state, State.CLIENT_HANDSHAKE_START)
        return client

    def create_server(self, alpn_protocols=None):
        configuration = QuicConfiguration(is_client=False)
        configuration.load_cert_chain(SERVER_CERTFILE, SERVER_KEYFILE)

        server = Context(
            alpn_protocols=alpn_protocols, is_client=False, max_early_data=0xFFFFFFFF
        )
        server.certificate = configuration.certificate
        server.certificate_private_key = configuration.private_key
        server.handshake_extensions = [
            (
                tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                SERVER_QUIC_TRANSPORT_PARAMETERS,
            )
        ]
        self.assertEqual(server.state, State.SERVER_EXPECT_CLIENT_HELLO)
        return server

    def test_client_unexpected_message(self):
        client = self.create_client()

        client.state = State.CLIENT_EXPECT_SERVER_HELLO
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

        client.state = State.CLIENT_EXPECT_ENCRYPTED_EXTENSIONS
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

        client.state = State.CLIENT_EXPECT_CERTIFICATE_REQUEST_OR_CERTIFICATE
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

        client.state = State.CLIENT_EXPECT_CERTIFICATE_VERIFY
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

        client.state = State.CLIENT_EXPECT_FINISHED
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

        client.state = State.CLIENT_POST_HANDSHAKE
        with self.assertRaises(tls.AlertUnexpectedMessage):
            client.handle_message(b"\x00\x00\x00\x00", create_buffers())

    def test_client_bad_certificate_verify_data(self):
        client = self.create_client()
        server = self.create_server()

        # send client hello
        client_buf = create_buffers()
        client.handle_message(b"", client_buf)
        self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
        server_input = merge_buffers(client_buf)
        reset_buffers(client_buf)

        # handle client hello
        # send server hello, encrypted extensions, certificate, certificate verify, finished
        server_buf = create_buffers()
        server.handle_message(server_input, server_buf)
        self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)
        client_input = merge_buffers(server_buf)
        reset_buffers(server_buf)

        # mess with certificate verify
        client_input = client_input[:-56] + bytes(4) + client_input[-52:]

        # handle server hello, encrypted extensions, certificate, certificate verify, finished
        with self.assertRaises(tls.AlertDecryptError):
            client.handle_message(client_input, client_buf)

    def test_client_bad_finished_verify_data(self):
        client = self.create_client()
        server = self.create_server()

        # send client hello
        client_buf = create_buffers()
        client.handle_message(b"", client_buf)
        self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
        server_input = merge_buffers(client_buf)
        reset_buffers(client_buf)

        # handle client hello
        # send server hello, encrypted extensions, certificate, certificate verify, finished
        server_buf = create_buffers()
        server.handle_message(server_input, server_buf)
        self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)
        client_input = merge_buffers(server_buf)
        reset_buffers(server_buf)

        # mess with finished verify data
        client_input = client_input[:-4] + bytes(4)

        # handle server hello, encrypted extensions, certificate, certificate verify, finished
        with self.assertRaises(tls.AlertDecryptError):
            client.handle_message(client_input, client_buf)

    def test_server_unexpected_message(self):
        server = self.create_server()

        server.state = State.SERVER_EXPECT_CLIENT_HELLO
        with self.assertRaises(tls.AlertUnexpectedMessage):
            server.handle_message(b"\x00\x00\x00\x00", create_buffers())

        server.state = State.SERVER_EXPECT_FINISHED
        with self.assertRaises(tls.AlertUnexpectedMessage):
            server.handle_message(b"\x00\x00\x00\x00", create_buffers())

        server.state = State.SERVER_POST_HANDSHAKE
        with self.assertRaises(tls.AlertUnexpectedMessage):
            server.handle_message(b"\x00\x00\x00\x00", create_buffers())

    def _server_fail_hello(self, client, server):
        # send client hello
        client_buf = create_buffers()
        client.handle_message(b"", client_buf)
        self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
        server_input = merge_buffers(client_buf)
        reset_buffers(client_buf)

        # handle client hello
        server_buf = create_buffers()
        server.handle_message(server_input, server_buf)

    def test_server_unsupported_cipher_suite(self):
        client = self.create_client()
        client._cipher_suites = [tls.CipherSuite.AES_128_GCM_SHA256]

        server = self.create_server()
        server._cipher_suites = [tls.CipherSuite.AES_256_GCM_SHA384]

        with self.assertRaises(tls.AlertHandshakeFailure) as cm:
            self._server_fail_hello(client, server)
        self.assertEqual(str(cm.exception), "No supported cipher suite")

    def test_server_unsupported_signature_algorithm(self):
        client = self.create_client()
        client._signature_algorithms = [tls.SignatureAlgorithm.ED448]

        server = self.create_server()

        with self.assertRaises(tls.AlertHandshakeFailure) as cm:
            self._server_fail_hello(client, server)
        self.assertEqual(str(cm.exception), "No supported signature algorithm")

    def test_server_unsupported_version(self):
        client = self.create_client()
        client._supported_versions = [tls.TLS_VERSION_1_2]

        server = self.create_server()

        with self.assertRaises(tls.AlertProtocolVersion) as cm:
            self._server_fail_hello(client, server)
        self.assertEqual(str(cm.exception), "No supported protocol version")

    def test_server_bad_finished_verify_data(self):
        client = self.create_client()
        server = self.create_server()

        # send client hello
        client_buf = create_buffers()
        client.handle_message(b"", client_buf)
        self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
        server_input = merge_buffers(client_buf)
        reset_buffers(client_buf)

        # handle client hello
        # send server hello, encrypted extensions, certificate, certificate verify, finished
        server_buf = create_buffers()
        server.handle_message(server_input, server_buf)
        self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)
        client_input = merge_buffers(server_buf)
        reset_buffers(server_buf)

        # handle server hello, encrypted extensions, certificate, certificate verify, finished
        # send finished
        client.handle_message(client_input, client_buf)
        self.assertEqual(client.state, State.CLIENT_POST_HANDSHAKE)
        server_input = merge_buffers(client_buf)
        reset_buffers(client_buf)

        # mess with finished verify data
        server_input = server_input[:-4] + bytes(4)

        # handle finished
        with self.assertRaises(tls.AlertDecryptError):
            server.handle_message(server_input, server_buf)

    def _handshake(self, client, server):
        # send client hello
        client_buf = create_buffers()
        client.handle_message(b"", client_buf)
        self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
        server_input = merge_buffers(client_buf)
        self.assertGreaterEqual(len(server_input), 213)
        self.assertLessEqual(len(server_input), 358)
        reset_buffers(client_buf)

        # handle client hello
        # send server hello, encrypted extensions, certificate, certificate verify, finished, (session ticket)
        server_buf = create_buffers()
        server.handle_message(server_input, server_buf)
        self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)
        client_input = merge_buffers(server_buf)
        self.assertGreaterEqual(len(client_input), 600)
        self.assertLessEqual(len(client_input), 2316)

        reset_buffers(server_buf)

        # handle server hello, encrypted extensions, certificate, certificate verify, finished, (session ticket)
        # send finished
        client.handle_message(client_input, client_buf)
        self.assertEqual(client.state, State.CLIENT_POST_HANDSHAKE)
        server_input = merge_buffers(client_buf)
        self.assertEqual(len(server_input), 52)
        reset_buffers(client_buf)

        # handle finished
        server.handle_message(server_input, server_buf)
        self.assertEqual(server.state, State.SERVER_POST_HANDSHAKE)
        client_input = merge_buffers(server_buf)
        self.assertEqual(len(client_input), 0)

        # check keys match
        self.assertEqual(client._dec_key, server._enc_key)
        self.assertEqual(client._enc_key, server._dec_key)

        # check cipher suite
        self.assertEqual(
            client.key_schedule.cipher_suite, tls.CipherSuite.AES_256_GCM_SHA384
        )
        self.assertEqual(
            server.key_schedule.cipher_suite, tls.CipherSuite.AES_256_GCM_SHA384
        )

    def test_handshake(self):
        client = self.create_client()
        server = self.create_server()

        self._handshake(client, server)

        # check ALPN matches
        self.assertEqual(client.alpn_negotiated, None)
        self.assertEqual(server.alpn_negotiated, None)

    def test_handshake_ecdsa_secp256r1(self):
        server = self.create_server()
        server.certificate, server.certificate_private_key = generate_ec_certificate(
            common_name="example.com", curve=ec.SECP256R1
        )

        client = self.create_client(
            cadata=server.certificate.public_bytes(serialization.Encoding.PEM),
            cafile=None,
        )

        self._handshake(client, server)

        # check ALPN matches
        self.assertEqual(client.alpn_negotiated, None)
        self.assertEqual(server.alpn_negotiated, None)

    def test_handshake_with_alpn(self):
        client = self.create_client(alpn_protocols=["hq-20"])
        server = self.create_server(alpn_protocols=["hq-20", "h3-20"])

        self._handshake(client, server)

        # check ALPN matches
        self.assertEqual(client.alpn_negotiated, "hq-20")
        self.assertEqual(server.alpn_negotiated, "hq-20")

    def test_handshake_with_alpn_fail(self):
        client = self.create_client(alpn_protocols=["hq-20"])
        server = self.create_server(alpn_protocols=["h3-20"])

        with self.assertRaises(tls.AlertHandshakeFailure) as cm:
            self._handshake(client, server)
        self.assertEqual(str(cm.exception), "No common ALPN protocols")

    def test_handshake_with_rsa_pkcs1_sha256_signature(self):
        client = self.create_client()
        client._signature_algorithms = [tls.SignatureAlgorithm.RSA_PKCS1_SHA256]
        server = self.create_server()

        self._handshake(client, server)

    def test_handshake_with_certificate_error(self):
        client = self.create_client(cafile=None)
        server = self.create_server()

        with self.assertRaises(tls.AlertBadCertificate) as cm:
            self._handshake(client, server)
        self.assertEqual(str(cm.exception), "unable to get local issuer certificate")

    def test_handshake_with_certificate_no_verify(self):
        client = self.create_client(cafile=None, verify_mode=ssl.CERT_NONE)
        server = self.create_server()

        self._handshake(client, server)

    def test_handshake_with_grease_group(self):
        client = self.create_client()
        client._supported_groups = [tls.Group.GREASE, tls.Group.SECP256R1]
        server = self.create_server()

        self._handshake(client, server)

    def test_handshake_with_x25519(self):
        client = self.create_client()
        client._supported_groups = [tls.Group.X25519]
        server = self.create_server()

        try:
            self._handshake(client, server)
        except UnsupportedAlgorithm as exc:
            self.skipTest(str(exc))

    def test_handshake_with_x448(self):
        client = self.create_client()
        client._supported_groups = [tls.Group.X448]
        server = self.create_server()

        try:
            self._handshake(client, server)
        except UnsupportedAlgorithm as exc:
            self.skipTest(str(exc))

    def test_session_ticket(self):
        client_tickets = []
        server_tickets = []

        def client_new_ticket(ticket):
            client_tickets.append(ticket)

        def server_get_ticket(label):
            for t in server_tickets:
                if t.ticket == label:
                    return t
            return None

        def server_new_ticket(ticket):
            server_tickets.append(ticket)

        def first_handshake():
            client = self.create_client()
            client.new_session_ticket_cb = client_new_ticket

            server = self.create_server()
            server.new_session_ticket_cb = server_new_ticket

            self._handshake(client, server)

            # check session resumption was not used
            self.assertFalse(client.session_resumed)
            self.assertFalse(server.session_resumed)

            # check tickets match
            self.assertEqual(len(client_tickets), 1)
            self.assertEqual(len(server_tickets), 1)
            self.assertEqual(client_tickets[0].ticket, server_tickets[0].ticket)
            self.assertEqual(
                client_tickets[0].resumption_secret, server_tickets[0].resumption_secret
            )

        def second_handshake():
            client = self.create_client()
            client.session_ticket = client_tickets[0]

            server = self.create_server()
            server.get_session_ticket_cb = server_get_ticket

            # send client hello with pre_shared_key
            client_buf = create_buffers()
            client.handle_message(b"", client_buf)
            self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
            server_input = merge_buffers(client_buf)
            self.assertGreaterEqual(len(server_input), 383)
            self.assertLessEqual(len(server_input), 483)
            reset_buffers(client_buf)

            # handle client hello
            # send server hello, encrypted extensions, finished
            server_buf = create_buffers()
            server.handle_message(server_input, server_buf)
            self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)
            client_input = merge_buffers(server_buf)
            self.assertEqual(len(client_input), 307)
            reset_buffers(server_buf)

            # handle server hello, encrypted extensions, certificate, certificate verify, finished
            # send finished
            client.handle_message(client_input, client_buf)
            self.assertEqual(client.state, State.CLIENT_POST_HANDSHAKE)
            server_input = merge_buffers(client_buf)
            self.assertEqual(len(server_input), 52)
            reset_buffers(client_buf)

            # handle finished
            # send new_session_ticket
            server.handle_message(server_input, server_buf)
            self.assertEqual(server.state, State.SERVER_POST_HANDSHAKE)
            client_input = merge_buffers(server_buf)
            self.assertEqual(len(client_input), 0)
            reset_buffers(server_buf)

            # check keys match
            self.assertEqual(client._dec_key, server._enc_key)
            self.assertEqual(client._enc_key, server._dec_key)

            # check session resumption was used
            self.assertTrue(client.session_resumed)
            self.assertTrue(server.session_resumed)

        def second_handshake_bad_binder():
            client = self.create_client()
            client.session_ticket = client_tickets[0]

            server = self.create_server()
            server.get_session_ticket_cb = server_get_ticket

            # send client hello with pre_shared_key
            client_buf = create_buffers()
            client.handle_message(b"", client_buf)
            self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
            server_input = merge_buffers(client_buf)
            self.assertGreaterEqual(len(server_input), 383)
            self.assertLessEqual(len(server_input), 483)
            reset_buffers(client_buf)

            # tamper with binder
            server_input = server_input[:-4] + bytes(4)

            # handle client hello
            # send server hello, encrypted extensions, finished
            server_buf = create_buffers()
            with self.assertRaises(tls.AlertHandshakeFailure) as cm:
                server.handle_message(server_input, server_buf)
            self.assertEqual(str(cm.exception), "PSK validation failed")

        def second_handshake_bad_pre_shared_key():
            client = self.create_client()
            client.session_ticket = client_tickets[0]

            server = self.create_server()
            server.get_session_ticket_cb = server_get_ticket

            # send client hello with pre_shared_key
            client_buf = create_buffers()
            client.handle_message(b"", client_buf)
            self.assertEqual(client.state, State.CLIENT_EXPECT_SERVER_HELLO)
            server_input = merge_buffers(client_buf)
            self.assertGreaterEqual(len(server_input), 383)
            self.assertLessEqual(len(server_input), 483)
            reset_buffers(client_buf)

            # handle client hello
            # send server hello, encrypted extensions, finished
            server_buf = create_buffers()
            server.handle_message(server_input, server_buf)
            self.assertEqual(server.state, State.SERVER_EXPECT_FINISHED)

            # tamper with pre_share_key index
            buf = server_buf[tls.Epoch.INITIAL]
            buf.seek(buf.tell() - 1)
            buf.push_uint8(1)
            client_input = merge_buffers(server_buf)
            self.assertEqual(len(client_input), 307)
            reset_buffers(server_buf)

            # handle server hello and bomb
            with self.assertRaises(tls.AlertIllegalParameter):
                client.handle_message(client_input, client_buf)

        first_handshake()
        second_handshake()
        second_handshake_bad_binder()
        second_handshake_bad_pre_shared_key()


class TlsTest(TestCase):
    def test_pull_client_hello(self):
        buf = Buffer(data=load("tls_client_hello.bin"))
        hello = pull_client_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello.random,
            binascii.unhexlify(
                "18b2b23bf3e44b5d52ccfe7aecbc5ff14eadc3d349fabf804d71f165ae76e7d5"
            ),
        )
        self.assertEqual(
            hello.session_id,
            binascii.unhexlify(
                "9aee82a2d186c1cb32a329d9dcfe004a1a438ad0485a53c6bfcf55c132a23235"
            ),
        )
        self.assertEqual(
            hello.cipher_suites,
            [
                tls.CipherSuite.AES_256_GCM_SHA384,
                tls.CipherSuite.AES_128_GCM_SHA256,
                tls.CipherSuite.CHACHA20_POLY1305_SHA256,
            ],
        )
        self.assertEqual(hello.compression_methods, [tls.CompressionMethod.NULL])

        # extensions
        self.assertEqual(hello.alpn_protocols, None)
        self.assertEqual(
            hello.key_share,
            [
                (
                    tls.Group.SECP256R1,
                    binascii.unhexlify(
                        "047bfea344467535054263b75def60cffa82405a211b68d1eb8d1d944e67aef8"
                        "93c7665a5473d032cfaf22a73da28eb4aacae0017ed12557b5791f98a1e84f15"
                        "b0"
                    ),
                )
            ],
        )
        self.assertEqual(
            hello.psk_key_exchange_modes, [tls.PskKeyExchangeMode.PSK_DHE_KE]
        )
        self.assertEqual(hello.server_name, None)
        self.assertEqual(
            hello.signature_algorithms,
            [
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256,
                tls.SignatureAlgorithm.ECDSA_SECP256R1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA1,
            ],
        )
        self.assertEqual(hello.supported_groups, [tls.Group.SECP256R1])
        self.assertEqual(
            hello.supported_versions,
            [
                tls.TLS_VERSION_1_3,
                tls.TLS_VERSION_1_3_DRAFT_28,
                tls.TLS_VERSION_1_3_DRAFT_27,
                tls.TLS_VERSION_1_3_DRAFT_26,
            ],
        )

        self.assertEqual(
            hello.other_extensions,
            [
                (
                    tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                    CLIENT_QUIC_TRANSPORT_PARAMETERS,
                )
            ],
        )

    def test_pull_client_hello_with_alpn(self):
        buf = Buffer(data=load("tls_client_hello_with_alpn.bin"))
        hello = pull_client_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello.random,
            binascii.unhexlify(
                "ed575c6fbd599c4dfaabd003dca6e860ccdb0e1782c1af02e57bf27cb6479b76"
            ),
        )
        self.assertEqual(hello.session_id, b"")
        self.assertEqual(
            hello.cipher_suites,
            [
                tls.CipherSuite.AES_128_GCM_SHA256,
                tls.CipherSuite.AES_256_GCM_SHA384,
                tls.CipherSuite.CHACHA20_POLY1305_SHA256,
                tls.CipherSuite.EMPTY_RENEGOTIATION_INFO_SCSV,
            ],
        )
        self.assertEqual(hello.compression_methods, [tls.CompressionMethod.NULL])

        # extensions
        self.assertEqual(hello.alpn_protocols, ["h3-19"])
        self.assertEqual(hello.early_data, False)
        self.assertEqual(
            hello.key_share,
            [
                (
                    tls.Group.SECP256R1,
                    binascii.unhexlify(
                        "048842315c437bb0ce2929c816fee4e942ec5cb6db6a6b9bf622680188ebb0d4"
                        "b652e69033f71686aa01cbc79155866e264c9f33f45aa16b0dfa10a222e3a669"
                        "22"
                    ),
                )
            ],
        )
        self.assertEqual(
            hello.psk_key_exchange_modes, [tls.PskKeyExchangeMode.PSK_DHE_KE]
        )
        self.assertEqual(hello.server_name, "cloudflare-quic.com")
        self.assertEqual(
            hello.signature_algorithms,
            [
                tls.SignatureAlgorithm.ECDSA_SECP256R1_SHA256,
                tls.SignatureAlgorithm.ECDSA_SECP384R1_SHA384,
                tls.SignatureAlgorithm.ECDSA_SECP521R1_SHA512,
                tls.SignatureAlgorithm.ED25519,
                tls.SignatureAlgorithm.ED448,
                tls.SignatureAlgorithm.RSA_PSS_PSS_SHA256,
                tls.SignatureAlgorithm.RSA_PSS_PSS_SHA384,
                tls.SignatureAlgorithm.RSA_PSS_PSS_SHA512,
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256,
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA384,
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA512,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA384,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA512,
            ],
        )
        self.assertEqual(
            hello.supported_groups,
            [
                tls.Group.SECP256R1,
                tls.Group.X25519,
                tls.Group.SECP384R1,
                tls.Group.SECP521R1,
            ],
        )
        self.assertEqual(hello.supported_versions, [tls.TLS_VERSION_1_3])

        # serialize
        buf = Buffer(1000)
        push_client_hello(buf, hello)
        self.assertEqual(len(buf.data), len(load("tls_client_hello_with_alpn.bin")))

    def test_pull_client_hello_with_psk(self):
        buf = Buffer(data=load("tls_client_hello_with_psk.bin"))
        hello = pull_client_hello(buf)

        self.assertEqual(hello.early_data, True)
        self.assertEqual(
            hello.pre_shared_key,
            tls.OfferedPsks(
                identities=[
                    (
                        binascii.unhexlify(
                            "fab3dc7d79f35ea53e9adf21150e601591a750b80cde0cd167fef6e0cdbc032a"
                            "c4161fc5c5b66679de49524bd5624c50d71ba3e650780a4bfe402d6a06a00525"
                            "0b5dc52085233b69d0dd13924cc5c713a396784ecafc59f5ea73c1585d79621b"
                            "8a94e4f2291b17427d5185abf4a994fca74ee7a7f993a950c71003fc7cf8"
                        ),
                        2067156378,
                    )
                ],
                binders=[
                    binascii.unhexlify(
                        "1788ad43fdff37cfc628f24b6ce7c8c76180705380da17da32811b5bae4e78"
                        "d7aaaf65a9b713872f2bb28818ca1a6b01"
                    )
                ],
            ),
        )

        self.assertTrue(buf.eof())

        # serialize
        buf = Buffer(1000)
        push_client_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_client_hello_with_psk.bin"))

    def test_pull_client_hello_with_sni(self):
        buf = Buffer(data=load("tls_client_hello_with_sni.bin"))
        hello = pull_client_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello.random,
            binascii.unhexlify(
                "987d8934140b0a42cc5545071f3f9f7f61963d7b6404eb674c8dbe513604346b"
            ),
        )
        self.assertEqual(
            hello.session_id,
            binascii.unhexlify(
                "26b19bdd30dbf751015a3a16e13bd59002dfe420b799d2a5cd5e11b8fa7bcb66"
            ),
        )
        self.assertEqual(
            hello.cipher_suites,
            [
                tls.CipherSuite.AES_256_GCM_SHA384,
                tls.CipherSuite.AES_128_GCM_SHA256,
                tls.CipherSuite.CHACHA20_POLY1305_SHA256,
            ],
        )
        self.assertEqual(hello.compression_methods, [tls.CompressionMethod.NULL])

        # extensions
        self.assertEqual(hello.alpn_protocols, None)
        self.assertEqual(
            hello.key_share,
            [
                (
                    tls.Group.SECP256R1,
                    binascii.unhexlify(
                        "04b62d70f907c814cd65d0f73b8b991f06b70c77153f548410a191d2b19764a2"
                        "ecc06065a480efa9e1f10c8da6e737d5bfc04be3f773e20a0c997f51b5621280"
                        "40"
                    ),
                )
            ],
        )
        self.assertEqual(
            hello.psk_key_exchange_modes, [tls.PskKeyExchangeMode.PSK_DHE_KE]
        )
        self.assertEqual(hello.server_name, "cloudflare-quic.com")
        self.assertEqual(
            hello.signature_algorithms,
            [
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256,
                tls.SignatureAlgorithm.ECDSA_SECP256R1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA1,
            ],
        )
        self.assertEqual(hello.supported_groups, [tls.Group.SECP256R1])
        self.assertEqual(
            hello.supported_versions,
            [
                tls.TLS_VERSION_1_3,
                tls.TLS_VERSION_1_3_DRAFT_28,
                tls.TLS_VERSION_1_3_DRAFT_27,
                tls.TLS_VERSION_1_3_DRAFT_26,
            ],
        )

        self.assertEqual(
            hello.other_extensions,
            [
                (
                    tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                    CLIENT_QUIC_TRANSPORT_PARAMETERS,
                )
            ],
        )

        # serialize
        buf = Buffer(1000)
        push_client_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_client_hello_with_sni.bin"))

    def test_push_client_hello(self):
        hello = ClientHello(
            random=binascii.unhexlify(
                "18b2b23bf3e44b5d52ccfe7aecbc5ff14eadc3d349fabf804d71f165ae76e7d5"
            ),
            session_id=binascii.unhexlify(
                "9aee82a2d186c1cb32a329d9dcfe004a1a438ad0485a53c6bfcf55c132a23235"
            ),
            cipher_suites=[
                tls.CipherSuite.AES_256_GCM_SHA384,
                tls.CipherSuite.AES_128_GCM_SHA256,
                tls.CipherSuite.CHACHA20_POLY1305_SHA256,
            ],
            compression_methods=[tls.CompressionMethod.NULL],
            key_share=[
                (
                    tls.Group.SECP256R1,
                    binascii.unhexlify(
                        "047bfea344467535054263b75def60cffa82405a211b68d1eb8d1d944e67aef8"
                        "93c7665a5473d032cfaf22a73da28eb4aacae0017ed12557b5791f98a1e84f15"
                        "b0"
                    ),
                )
            ],
            psk_key_exchange_modes=[tls.PskKeyExchangeMode.PSK_DHE_KE],
            signature_algorithms=[
                tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256,
                tls.SignatureAlgorithm.ECDSA_SECP256R1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA256,
                tls.SignatureAlgorithm.RSA_PKCS1_SHA1,
            ],
            supported_groups=[tls.Group.SECP256R1],
            supported_versions=[
                tls.TLS_VERSION_1_3,
                tls.TLS_VERSION_1_3_DRAFT_28,
                tls.TLS_VERSION_1_3_DRAFT_27,
                tls.TLS_VERSION_1_3_DRAFT_26,
            ],
            other_extensions=[
                (
                    tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                    CLIENT_QUIC_TRANSPORT_PARAMETERS,
                )
            ],
        )

        buf = Buffer(1000)
        push_client_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_client_hello.bin"))

    def test_pull_server_hello(self):
        buf = Buffer(data=load("tls_server_hello.bin"))
        hello = pull_server_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello.random,
            binascii.unhexlify(
                "ada85271d19680c615ea7336519e3fdf6f1e26f3b1075ee1de96ffa8884e8280"
            ),
        )
        self.assertEqual(
            hello.session_id,
            binascii.unhexlify(
                "9aee82a2d186c1cb32a329d9dcfe004a1a438ad0485a53c6bfcf55c132a23235"
            ),
        )
        self.assertEqual(hello.cipher_suite, tls.CipherSuite.AES_256_GCM_SHA384)
        self.assertEqual(hello.compression_method, tls.CompressionMethod.NULL)
        self.assertEqual(
            hello.key_share,
            (
                tls.Group.SECP256R1,
                binascii.unhexlify(
                    "048b27d0282242d84b7fcc02a9c4f13eca0329e3c7029aa34a33794e6e7ba189"
                    "5cca1c503bf0378ac6937c354912116ff3251026bca1958d7f387316c83ae6cf"
                    "b2"
                ),
            ),
        )
        self.assertEqual(hello.pre_shared_key, None)
        self.assertEqual(hello.supported_version, tls.TLS_VERSION_1_3)

    def test_pull_server_hello_with_psk(self):
        buf = Buffer(data=load("tls_server_hello_with_psk.bin"))
        hello = pull_server_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello.random,
            binascii.unhexlify(
                "ccbaaf04fc1bd5143b2cc6b97520cf37d91470dbfc8127131a7bf0f941e3a137"
            ),
        )
        self.assertEqual(
            hello.session_id,
            binascii.unhexlify(
                "9483e7e895d0f4cec17086b0849601c0632662cd764e828f2f892f4c4b7771b0"
            ),
        )
        self.assertEqual(hello.cipher_suite, tls.CipherSuite.AES_256_GCM_SHA384)
        self.assertEqual(hello.compression_method, tls.CompressionMethod.NULL)
        self.assertEqual(
            hello.key_share,
            (
                tls.Group.SECP256R1,
                binascii.unhexlify(
                    "0485d7cecbebfc548fc657bf51b8e8da842a4056b164a27f7702ca318c16e488"
                    "18b6409593b15c6649d6f459387a53128b164178adc840179aad01d36ce95d62"
                    "76"
                ),
            ),
        )
        self.assertEqual(hello.pre_shared_key, 0)
        self.assertEqual(hello.supported_version, tls.TLS_VERSION_1_3)

        # serialize
        buf = Buffer(1000)
        push_server_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_server_hello_with_psk.bin"))

    def test_pull_server_hello_with_unknown_extension(self):
        buf = Buffer(data=load("tls_server_hello_with_unknown_extension.bin"))
        hello = pull_server_hello(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            hello,
            ServerHello(
                random=binascii.unhexlify(
                    "ada85271d19680c615ea7336519e3fdf6f1e26f3b1075ee1de96ffa8884e8280"
                ),
                session_id=binascii.unhexlify(
                    "9aee82a2d186c1cb32a329d9dcfe004a1a438ad0485a53c6bfcf55c132a23235"
                ),
                cipher_suite=tls.CipherSuite.AES_256_GCM_SHA384,
                compression_method=tls.CompressionMethod.NULL,
                key_share=(
                    tls.Group.SECP256R1,
                    binascii.unhexlify(
                        "048b27d0282242d84b7fcc02a9c4f13eca0329e3c7029aa34a33794e6e7ba189"
                        "5cca1c503bf0378ac6937c354912116ff3251026bca1958d7f387316c83ae6cf"
                        "b2"
                    ),
                ),
                supported_version=tls.TLS_VERSION_1_3,
                other_extensions=[(12345, b"foo")],
            ),
        )

        # serialize
        buf = Buffer(1000)
        push_server_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_server_hello_with_unknown_extension.bin"))

    def test_push_server_hello(self):
        hello = ServerHello(
            random=binascii.unhexlify(
                "ada85271d19680c615ea7336519e3fdf6f1e26f3b1075ee1de96ffa8884e8280"
            ),
            session_id=binascii.unhexlify(
                "9aee82a2d186c1cb32a329d9dcfe004a1a438ad0485a53c6bfcf55c132a23235"
            ),
            cipher_suite=tls.CipherSuite.AES_256_GCM_SHA384,
            compression_method=tls.CompressionMethod.NULL,
            key_share=(
                tls.Group.SECP256R1,
                binascii.unhexlify(
                    "048b27d0282242d84b7fcc02a9c4f13eca0329e3c7029aa34a33794e6e7ba189"
                    "5cca1c503bf0378ac6937c354912116ff3251026bca1958d7f387316c83ae6cf"
                    "b2"
                ),
            ),
            supported_version=tls.TLS_VERSION_1_3,
        )

        buf = Buffer(1000)
        push_server_hello(buf, hello)
        self.assertEqual(buf.data, load("tls_server_hello.bin"))

    def test_pull_new_session_ticket(self):
        buf = Buffer(data=load("tls_new_session_ticket.bin"))
        new_session_ticket = pull_new_session_ticket(buf)
        self.assertIsNotNone(new_session_ticket)
        self.assertTrue(buf.eof())

        self.assertEqual(
            new_session_ticket,
            NewSessionTicket(
                ticket_lifetime=86400,
                ticket_age_add=3303452425,
                ticket_nonce=b"",
                ticket=binascii.unhexlify(
                    "dbe6f1a77a78c0426bfa607cd0d02b350247d90618704709596beda7e962cc81"
                ),
                max_early_data_size=0xFFFFFFFF,
            ),
        )

        # serialize
        buf = Buffer(100)
        push_new_session_ticket(buf, new_session_ticket)
        self.assertEqual(buf.data, load("tls_new_session_ticket.bin"))

    def test_pull_new_session_ticket_with_unknown_extension(self):
        buf = Buffer(data=load("tls_new_session_ticket_with_unknown_extension.bin"))
        new_session_ticket = pull_new_session_ticket(buf)
        self.assertIsNotNone(new_session_ticket)
        self.assertTrue(buf.eof())

        self.assertEqual(
            new_session_ticket,
            NewSessionTicket(
                ticket_lifetime=86400,
                ticket_age_add=3303452425,
                ticket_nonce=b"",
                ticket=binascii.unhexlify(
                    "dbe6f1a77a78c0426bfa607cd0d02b350247d90618704709596beda7e962cc81"
                ),
                max_early_data_size=0xFFFFFFFF,
                other_extensions=[(12345, b"foo")],
            ),
        )

        # serialize
        buf = Buffer(100)
        push_new_session_ticket(buf, new_session_ticket)
        self.assertEqual(
            buf.data, load("tls_new_session_ticket_with_unknown_extension.bin")
        )

    def test_encrypted_extensions(self):
        data = load("tls_encrypted_extensions.bin")
        buf = Buffer(data=data)
        extensions = pull_encrypted_extensions(buf)
        self.assertIsNotNone(extensions)
        self.assertTrue(buf.eof())

        self.assertEqual(
            extensions,
            EncryptedExtensions(
                other_extensions=[
                    (
                        tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                        SERVER_QUIC_TRANSPORT_PARAMETERS,
                    )
                ]
            ),
        )

        # serialize
        buf = Buffer(capacity=100)
        push_encrypted_extensions(buf, extensions)
        self.assertEqual(buf.data, data)

    def test_encrypted_extensions_with_alpn(self):
        data = load("tls_encrypted_extensions_with_alpn.bin")
        buf = Buffer(data=data)
        extensions = pull_encrypted_extensions(buf)
        self.assertIsNotNone(extensions)
        self.assertTrue(buf.eof())

        self.assertEqual(
            extensions,
            EncryptedExtensions(
                alpn_protocol="hq-20",
                other_extensions=[
                    (tls.ExtensionType.SERVER_NAME, b""),
                    (
                        tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                        SERVER_QUIC_TRANSPORT_PARAMETERS_2,
                    ),
                ],
            ),
        )

        # serialize
        buf = Buffer(115)
        push_encrypted_extensions(buf, extensions)
        self.assertTrue(buf.eof())

    def test_pull_encrypted_extensions_with_alpn_and_early_data(self):
        buf = Buffer(data=load("tls_encrypted_extensions_with_alpn_and_early_data.bin"))
        extensions = pull_encrypted_extensions(buf)
        self.assertIsNotNone(extensions)
        self.assertTrue(buf.eof())

        self.assertEqual(
            extensions,
            EncryptedExtensions(
                alpn_protocol="hq-20",
                early_data=True,
                other_extensions=[
                    (tls.ExtensionType.SERVER_NAME, b""),
                    (
                        tls.ExtensionType.QUIC_TRANSPORT_PARAMETERS,
                        SERVER_QUIC_TRANSPORT_PARAMETERS_3,
                    ),
                ],
            ),
        )

        # serialize
        buf = Buffer(116)
        push_encrypted_extensions(buf, extensions)
        self.assertTrue(buf.eof())

    def test_pull_certificate(self):
        buf = Buffer(data=load("tls_certificate.bin"))
        certificate = pull_certificate(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(certificate.request_context, b"")
        self.assertEqual(certificate.certificates, [(CERTIFICATE_DATA, b"")])

    def test_push_certificate(self):
        certificate = Certificate(
            request_context=b"", certificates=[(CERTIFICATE_DATA, b"")]
        )

        buf = Buffer(1600)
        push_certificate(buf, certificate)
        self.assertEqual(buf.data, load("tls_certificate.bin"))

    def test_pull_certificate_verify(self):
        buf = Buffer(data=load("tls_certificate_verify.bin"))
        verify = pull_certificate_verify(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(verify.algorithm, tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256)
        self.assertEqual(verify.signature, CERTIFICATE_VERIFY_SIGNATURE)

    def test_push_certificate_verify(self):
        verify = CertificateVerify(
            algorithm=tls.SignatureAlgorithm.RSA_PSS_RSAE_SHA256,
            signature=CERTIFICATE_VERIFY_SIGNATURE,
        )

        buf = Buffer(400)
        push_certificate_verify(buf, verify)
        self.assertEqual(buf.data, load("tls_certificate_verify.bin"))

    def test_pull_finished(self):
        buf = Buffer(data=load("tls_finished.bin"))
        finished = pull_finished(buf)
        self.assertTrue(buf.eof())

        self.assertEqual(
            finished.verify_data,
            binascii.unhexlify(
                "f157923234ff9a4921aadb2e0ec7b1a30fce73fb9ec0c4276f9af268f408ec68"
            ),
        )

    def test_push_finished(self):
        finished = Finished(
            verify_data=binascii.unhexlify(
                "f157923234ff9a4921aadb2e0ec7b1a30fce73fb9ec0c4276f9af268f408ec68"
            )
        )

        buf = Buffer(128)
        push_finished(buf, finished)
        self.assertEqual(buf.data, load("tls_finished.bin"))


class VerifyCertificateTest(TestCase):
    def test_verify_certificate_chain(self):
        with open(SERVER_CERTFILE, "rb") as fp:
            certificate = load_pem_x509_certificates(fp.read())[0]

        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_before

            # fail
            with self.assertRaises(tls.AlertBadCertificate) as cm:
                verify_certificate(certificate=certificate, server_name="localhost")
            self.assertEqual(
                str(cm.exception), "unable to get local issuer certificate"
            )

            # ok
            verify_certificate(
                cafile=SERVER_CACERTFILE,
                certificate=certificate,
                server_name="localhost",
            )

    def test_verify_certificate_chain_self_signed(self):
        certificate, _ = generate_ec_certificate(
            common_name="localhost", curve=ec.SECP256R1
        )

        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_before

            # fail
            with self.assertRaises(tls.AlertBadCertificate) as cm:
                verify_certificate(certificate=certificate, server_name="localhost")
            self.assertEqual(str(cm.exception), "self signed certificate")

            # ok
            verify_certificate(
                cadata=certificate.public_bytes(serialization.Encoding.PEM),
                certificate=certificate,
                server_name="localhost",
            )

    @patch("aioquic.tls.lib.X509_STORE_new")
    def test_verify_certificate_chain_internal_error(self, mock_store_new):
        mock_store_new.return_value = tls.ffi.NULL

        certificate, _ = generate_ec_certificate(
            common_name="localhost", curve=ec.SECP256R1
        )

        with self.assertRaises(tls.AlertInternalError) as cm:
            verify_certificate(
                cadata=certificate.public_bytes(serialization.Encoding.PEM),
                certificate=certificate,
                server_name="localhost",
            )
        self.assertEqual(str(cm.exception), "OpenSSL call to X509_store_new failed")

    def test_verify_dates(self):
        certificate, _ = generate_ec_certificate(
            common_name="example.com", curve=ec.SECP256R1
        )
        cadata = certificate.public_bytes(serialization.Encoding.PEM)

        #  too early
        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = (
                certificate.not_valid_before - datetime.timedelta(seconds=1)
            )
            with self.assertRaises(tls.AlertCertificateExpired) as cm:
                verify_certificate(
                    cadata=cadata, certificate=certificate, server_name="example.com"
                )
            self.assertEqual(str(cm.exception), "Certificate is not valid yet")

        # valid
        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_before
            verify_certificate(
                cadata=cadata, certificate=certificate, server_name="example.com"
            )

        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_after
            verify_certificate(
                cadata=cadata, certificate=certificate, server_name="example.com"
            )

        # too late
        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_after + datetime.timedelta(
                seconds=1
            )
            with self.assertRaises(tls.AlertCertificateExpired) as cm:
                verify_certificate(
                    cadata=cadata, certificate=certificate, server_name="example.com"
                )
            self.assertEqual(str(cm.exception), "Certificate is no longer valid")

    def test_verify_subject(self):
        certificate, _ = generate_ec_certificate(
            common_name="example.com", curve=ec.SECP256R1
        )
        cadata = certificate.public_bytes(serialization.Encoding.PEM)

        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_before

            # valid
            verify_certificate(
                cadata=cadata, certificate=certificate, server_name="example.com"
            )

            # invalid
            with self.assertRaises(tls.AlertBadCertificate) as cm:
                verify_certificate(
                    cadata=cadata,
                    certificate=certificate,
                    server_name="test.example.com",
                )
            self.assertEqual(
                str(cm.exception),
                "hostname 'test.example.com' doesn't match 'example.com'",
            )

            with self.assertRaises(tls.AlertBadCertificate) as cm:
                verify_certificate(
                    cadata=cadata, certificate=certificate, server_name="acme.com"
                )
            self.assertEqual(
                str(cm.exception), "hostname 'acme.com' doesn't match 'example.com'"
            )

    def test_verify_subject_with_subjaltname(self):
        certificate, _ = generate_ec_certificate(
            alternative_names=["*.example.com", "example.com"],
            common_name="example.com",
            curve=ec.SECP256R1,
        )
        cadata = certificate.public_bytes(serialization.Encoding.PEM)

        with patch("aioquic.tls.utcnow") as mock_utcnow:
            mock_utcnow.return_value = certificate.not_valid_before

            # valid
            verify_certificate(
                cadata=cadata, certificate=certificate, server_name="example.com"
            )
            verify_certificate(
                cadata=cadata, certificate=certificate, server_name="test.example.com"
            )

            # invalid
            with self.assertRaises(tls.AlertBadCertificate) as cm:
                verify_certificate(
                    cadata=cadata, certificate=certificate, server_name="acme.com"
                )
            self.assertEqual(
                str(cm.exception),
                "hostname 'acme.com' doesn't match either of '*.example.com', 'example.com'",
            )
