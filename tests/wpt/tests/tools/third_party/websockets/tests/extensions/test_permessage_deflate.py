import dataclasses
import unittest

from websockets.exceptions import (
    DuplicateParameter,
    InvalidParameterName,
    InvalidParameterValue,
    NegotiationError,
    PayloadTooBig,
    ProtocolError,
)
from websockets.extensions.permessage_deflate import *
from websockets.frames import (
    OP_BINARY,
    OP_CLOSE,
    OP_CONT,
    OP_PING,
    OP_PONG,
    OP_TEXT,
    Close,
    CloseCode,
    Frame,
)

from .utils import ClientNoOpExtensionFactory, ServerNoOpExtensionFactory


class PerMessageDeflateTestsMixin:
    def assertExtensionEqual(self, extension1, extension2):
        self.assertEqual(
            extension1.remote_no_context_takeover,
            extension2.remote_no_context_takeover,
        )
        self.assertEqual(
            extension1.local_no_context_takeover,
            extension2.local_no_context_takeover,
        )
        self.assertEqual(
            extension1.remote_max_window_bits,
            extension2.remote_max_window_bits,
        )
        self.assertEqual(
            extension1.local_max_window_bits,
            extension2.local_max_window_bits,
        )


class PerMessageDeflateTests(unittest.TestCase, PerMessageDeflateTestsMixin):
    def setUp(self):
        # Set up an instance of the permessage-deflate extension with the most
        # common settings. Since the extension is symmetrical, this instance
        # may be used for testing both encoding and decoding.
        self.extension = PerMessageDeflate(False, False, 15, 15)

    def test_name(self):
        assert self.extension.name == "permessage-deflate"

    def test_repr(self):
        self.assertExtensionEqual(eval(repr(self.extension)), self.extension)

    # Control frames aren't encoded or decoded.

    def test_no_encode_decode_ping_frame(self):
        frame = Frame(OP_PING, b"")

        self.assertEqual(self.extension.encode(frame), frame)

        self.assertEqual(self.extension.decode(frame), frame)

    def test_no_encode_decode_pong_frame(self):
        frame = Frame(OP_PONG, b"")

        self.assertEqual(self.extension.encode(frame), frame)

        self.assertEqual(self.extension.decode(frame), frame)

    def test_no_encode_decode_close_frame(self):
        frame = Frame(OP_CLOSE, Close(CloseCode.NORMAL_CLOSURE, "").serialize())

        self.assertEqual(self.extension.encode(frame), frame)

        self.assertEqual(self.extension.decode(frame), frame)

    # Data frames are encoded and decoded.

    def test_encode_decode_text_frame(self):
        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        enc_frame = self.extension.encode(frame)

        self.assertEqual(
            enc_frame,
            dataclasses.replace(frame, rsv1=True, data=b"JNL;\xbc\x12\x00"),
        )

        dec_frame = self.extension.decode(enc_frame)

        self.assertEqual(dec_frame, frame)

    def test_encode_decode_binary_frame(self):
        frame = Frame(OP_BINARY, b"tea")

        enc_frame = self.extension.encode(frame)

        self.assertEqual(
            enc_frame,
            dataclasses.replace(frame, rsv1=True, data=b"*IM\x04\x00"),
        )

        dec_frame = self.extension.decode(enc_frame)

        self.assertEqual(dec_frame, frame)

    def test_encode_decode_fragmented_text_frame(self):
        frame1 = Frame(OP_TEXT, "café".encode("utf-8"), fin=False)
        frame2 = Frame(OP_CONT, " & ".encode("utf-8"), fin=False)
        frame3 = Frame(OP_CONT, "croissants".encode("utf-8"))

        enc_frame1 = self.extension.encode(frame1)
        enc_frame2 = self.extension.encode(frame2)
        enc_frame3 = self.extension.encode(frame3)

        self.assertEqual(
            enc_frame1,
            dataclasses.replace(
                frame1, rsv1=True, data=b"JNL;\xbc\x12\x00\x00\x00\xff\xff"
            ),
        )
        self.assertEqual(
            enc_frame2,
            dataclasses.replace(frame2, data=b"RPS\x00\x00\x00\x00\xff\xff"),
        )
        self.assertEqual(
            enc_frame3,
            dataclasses.replace(frame3, data=b"J.\xca\xcf,.N\xcc+)\x06\x00"),
        )

        dec_frame1 = self.extension.decode(enc_frame1)
        dec_frame2 = self.extension.decode(enc_frame2)
        dec_frame3 = self.extension.decode(enc_frame3)

        self.assertEqual(dec_frame1, frame1)
        self.assertEqual(dec_frame2, frame2)
        self.assertEqual(dec_frame3, frame3)

    def test_encode_decode_fragmented_binary_frame(self):
        frame1 = Frame(OP_TEXT, b"tea ", fin=False)
        frame2 = Frame(OP_CONT, b"time")

        enc_frame1 = self.extension.encode(frame1)
        enc_frame2 = self.extension.encode(frame2)

        self.assertEqual(
            enc_frame1,
            dataclasses.replace(
                frame1, rsv1=True, data=b"*IMT\x00\x00\x00\x00\xff\xff"
            ),
        )
        self.assertEqual(
            enc_frame2,
            dataclasses.replace(frame2, data=b"*\xc9\xccM\x05\x00"),
        )

        dec_frame1 = self.extension.decode(enc_frame1)
        dec_frame2 = self.extension.decode(enc_frame2)

        self.assertEqual(dec_frame1, frame1)
        self.assertEqual(dec_frame2, frame2)

    def test_no_decode_text_frame(self):
        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        # Try decoding a frame that wasn't encoded.
        self.assertEqual(self.extension.decode(frame), frame)

    def test_no_decode_binary_frame(self):
        frame = Frame(OP_TEXT, b"tea")

        # Try decoding a frame that wasn't encoded.
        self.assertEqual(self.extension.decode(frame), frame)

    def test_no_decode_fragmented_text_frame(self):
        frame1 = Frame(OP_TEXT, "café".encode("utf-8"), fin=False)
        frame2 = Frame(OP_CONT, " & ".encode("utf-8"), fin=False)
        frame3 = Frame(OP_CONT, "croissants".encode("utf-8"))

        dec_frame1 = self.extension.decode(frame1)
        dec_frame2 = self.extension.decode(frame2)
        dec_frame3 = self.extension.decode(frame3)

        self.assertEqual(dec_frame1, frame1)
        self.assertEqual(dec_frame2, frame2)
        self.assertEqual(dec_frame3, frame3)

    def test_no_decode_fragmented_binary_frame(self):
        frame1 = Frame(OP_TEXT, b"tea ", fin=False)
        frame2 = Frame(OP_CONT, b"time")

        dec_frame1 = self.extension.decode(frame1)
        dec_frame2 = self.extension.decode(frame2)

        self.assertEqual(dec_frame1, frame1)
        self.assertEqual(dec_frame2, frame2)

    def test_context_takeover(self):
        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        enc_frame1 = self.extension.encode(frame)
        enc_frame2 = self.extension.encode(frame)

        self.assertEqual(enc_frame1.data, b"JNL;\xbc\x12\x00")
        self.assertEqual(enc_frame2.data, b"J\x06\x11\x00\x00")

    def test_remote_no_context_takeover(self):
        # No context takeover when decoding messages.
        self.extension = PerMessageDeflate(True, False, 15, 15)

        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        enc_frame1 = self.extension.encode(frame)
        enc_frame2 = self.extension.encode(frame)

        self.assertEqual(enc_frame1.data, b"JNL;\xbc\x12\x00")
        self.assertEqual(enc_frame2.data, b"J\x06\x11\x00\x00")

        dec_frame1 = self.extension.decode(enc_frame1)
        self.assertEqual(dec_frame1, frame)

        with self.assertRaises(ProtocolError):
            self.extension.decode(enc_frame2)

    def test_local_no_context_takeover(self):
        # No context takeover when encoding and decoding messages.
        self.extension = PerMessageDeflate(True, True, 15, 15)

        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        enc_frame1 = self.extension.encode(frame)
        enc_frame2 = self.extension.encode(frame)

        self.assertEqual(enc_frame1.data, b"JNL;\xbc\x12\x00")
        self.assertEqual(enc_frame2.data, b"JNL;\xbc\x12\x00")

        dec_frame1 = self.extension.decode(enc_frame1)
        dec_frame2 = self.extension.decode(enc_frame2)

        self.assertEqual(dec_frame1, frame)
        self.assertEqual(dec_frame2, frame)

    # Compression settings can be customized.

    def test_compress_settings(self):
        # Configure an extension so that no compression actually occurs.
        extension = PerMessageDeflate(False, False, 15, 15, {"level": 0})

        frame = Frame(OP_TEXT, "café".encode("utf-8"))

        enc_frame = extension.encode(frame)

        self.assertEqual(
            enc_frame,
            dataclasses.replace(
                frame,
                rsv1=True,
                data=b"\x00\x05\x00\xfa\xffcaf\xc3\xa9\x00",  # not compressed
            ),
        )

    # Frames aren't decoded beyond max_size.

    def test_decompress_max_size(self):
        frame = Frame(OP_TEXT, ("a" * 20).encode("utf-8"))

        enc_frame = self.extension.encode(frame)

        self.assertEqual(enc_frame.data, b"JL\xc4\x04\x00\x00")

        with self.assertRaises(PayloadTooBig):
            self.extension.decode(enc_frame, max_size=10)


class ClientPerMessageDeflateFactoryTests(
    unittest.TestCase, PerMessageDeflateTestsMixin
):
    def test_name(self):
        assert ClientPerMessageDeflateFactory.name == "permessage-deflate"

    def test_init(self):
        for config in [
            (False, False, 8, None),  # server_max_window_bits ≥ 8
            (False, True, 15, None),  # server_max_window_bits ≤ 15
            (True, False, None, 8),  # client_max_window_bits ≥ 8
            (True, True, None, 15),  # client_max_window_bits ≤ 15
            (False, False, None, True),  # client_max_window_bits
            (False, False, None, None, {"memLevel": 4}),
        ]:
            with self.subTest(config=config):
                # This does not raise an exception.
                ClientPerMessageDeflateFactory(*config)

    def test_init_error(self):
        for config in [
            (False, False, 7, 8),  # server_max_window_bits < 8
            (False, True, 8, 7),  # client_max_window_bits < 8
            (True, False, 16, 15),  # server_max_window_bits > 15
            (True, True, 15, 16),  # client_max_window_bits > 15
            (False, False, True, None),  # server_max_window_bits
            (False, False, None, None, {"wbits": 11}),
        ]:
            with self.subTest(config=config):
                with self.assertRaises(ValueError):
                    ClientPerMessageDeflateFactory(*config)

    def test_get_request_params(self):
        for config, result in [
            # Test without any parameter
            (
                (False, False, None, None),
                [],
            ),
            # Test server_no_context_takeover
            (
                (True, False, None, None),
                [("server_no_context_takeover", None)],
            ),
            # Test client_no_context_takeover
            (
                (False, True, None, None),
                [("client_no_context_takeover", None)],
            ),
            # Test server_max_window_bits
            (
                (False, False, 10, None),
                [("server_max_window_bits", "10")],
            ),
            # Test client_max_window_bits
            (
                (False, False, None, 10),
                [("client_max_window_bits", "10")],
            ),
            (
                (False, False, None, True),
                [("client_max_window_bits", None)],
            ),
            # Test all parameters together
            (
                (True, True, 12, 12),
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "12"),
                    ("client_max_window_bits", "12"),
                ],
            ),
        ]:
            with self.subTest(config=config):
                factory = ClientPerMessageDeflateFactory(*config)
                self.assertEqual(factory.get_request_params(), result)

    def test_process_response_params(self):
        for config, response_params, result in [
            # Test without any parameter
            (
                (False, False, None, None),
                [],
                (False, False, 15, 15),
            ),
            (
                (False, False, None, None),
                [("unknown", None)],
                InvalidParameterName,
            ),
            # Test server_no_context_takeover
            (
                (False, False, None, None),
                [("server_no_context_takeover", None)],
                (True, False, 15, 15),
            ),
            (
                (True, False, None, None),
                [],
                NegotiationError,
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", None)],
                (True, False, 15, 15),
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", None)] * 2,
                DuplicateParameter,
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", "42")],
                InvalidParameterValue,
            ),
            # Test client_no_context_takeover
            (
                (False, False, None, None),
                [("client_no_context_takeover", None)],
                (False, True, 15, 15),
            ),
            (
                (False, True, None, None),
                [],
                (False, True, 15, 15),
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", None)],
                (False, True, 15, 15),
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", None)] * 2,
                DuplicateParameter,
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", "42")],
                InvalidParameterValue,
            ),
            # Test server_max_window_bits
            (
                (False, False, None, None),
                [("server_max_window_bits", "7")],
                NegotiationError,
            ),
            (
                (False, False, None, None),
                [("server_max_window_bits", "10")],
                (False, False, 10, 15),
            ),
            (
                (False, False, None, None),
                [("server_max_window_bits", "16")],
                NegotiationError,
            ),
            (
                (False, False, 12, None),
                [],
                NegotiationError,
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "10")],
                (False, False, 10, 15),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "12")],
                (False, False, 12, 15),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "13")],
                NegotiationError,
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "12")] * 2,
                DuplicateParameter,
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "42")],
                InvalidParameterValue,
            ),
            # Test client_max_window_bits
            (
                (False, False, None, None),
                [("client_max_window_bits", "10")],
                NegotiationError,
            ),
            (
                (False, False, None, True),
                [],
                (False, False, 15, 15),
            ),
            (
                (False, False, None, True),
                [("client_max_window_bits", "7")],
                NegotiationError,
            ),
            (
                (False, False, None, True),
                [("client_max_window_bits", "10")],
                (False, False, 15, 10),
            ),
            (
                (False, False, None, True),
                [("client_max_window_bits", "16")],
                NegotiationError,
            ),
            (
                (False, False, None, 12),
                [],
                (False, False, 15, 12),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "10")],
                (False, False, 15, 10),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "12")],
                (False, False, 15, 12),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "13")],
                NegotiationError,
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "12")] * 2,
                DuplicateParameter,
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "42")],
                InvalidParameterValue,
            ),
            # Test all parameters together
            (
                (True, True, 12, 12),
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                (True, True, 10, 10),
            ),
            (
                (False, False, None, True),
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                (True, True, 10, 10),
            ),
            (
                (True, True, 12, 12),
                [
                    ("server_no_context_takeover", None),
                    ("server_max_window_bits", "12"),
                ],
                (True, True, 12, 12),
            ),
        ]:
            with self.subTest(config=config, response_params=response_params):
                factory = ClientPerMessageDeflateFactory(*config)
                if isinstance(result, type) and issubclass(result, Exception):
                    with self.assertRaises(result):
                        factory.process_response_params(response_params, [])
                else:
                    extension = factory.process_response_params(response_params, [])
                    expected = PerMessageDeflate(*result)
                    self.assertExtensionEqual(extension, expected)

    def test_process_response_params_deduplication(self):
        factory = ClientPerMessageDeflateFactory(False, False, None, None)
        with self.assertRaises(NegotiationError):
            factory.process_response_params(
                [], [PerMessageDeflate(False, False, 15, 15)]
            )

    def test_enable_client_permessage_deflate(self):
        for extensions, (
            expected_len,
            expected_position,
            expected_compress_settings,
        ) in [
            (
                None,
                (1, 0, {"memLevel": 5}),
            ),
            (
                [],
                (1, 0, {"memLevel": 5}),
            ),
            (
                [ClientNoOpExtensionFactory()],
                (2, 1, {"memLevel": 5}),
            ),
            (
                [ClientPerMessageDeflateFactory(compress_settings={"memLevel": 7})],
                (1, 0, {"memLevel": 7}),
            ),
            (
                [
                    ClientPerMessageDeflateFactory(compress_settings={"memLevel": 7}),
                    ClientNoOpExtensionFactory(),
                ],
                (2, 0, {"memLevel": 7}),
            ),
            (
                [
                    ClientNoOpExtensionFactory(),
                    ClientPerMessageDeflateFactory(compress_settings={"memLevel": 7}),
                ],
                (2, 1, {"memLevel": 7}),
            ),
        ]:
            with self.subTest(extensions=extensions):
                extensions = enable_client_permessage_deflate(extensions)
                self.assertEqual(len(extensions), expected_len)
                extension = extensions[expected_position]
                self.assertIsInstance(extension, ClientPerMessageDeflateFactory)
                self.assertEqual(
                    extension.compress_settings,
                    expected_compress_settings,
                )


class ServerPerMessageDeflateFactoryTests(
    unittest.TestCase, PerMessageDeflateTestsMixin
):
    def test_name(self):
        assert ServerPerMessageDeflateFactory.name == "permessage-deflate"

    def test_init(self):
        for config in [
            (False, False, 8, None),  # server_max_window_bits ≥ 8
            (False, True, 15, None),  # server_max_window_bits ≤ 15
            (True, False, None, 8),  # client_max_window_bits ≥ 8
            (True, True, None, 15),  # client_max_window_bits ≤ 15
            (False, False, None, None, {"memLevel": 4}),
            (False, False, None, 12, {}, True),  # require_client_max_window_bits
        ]:
            with self.subTest(config=config):
                # This does not raise an exception.
                ServerPerMessageDeflateFactory(*config)

    def test_init_error(self):
        for config in [
            (False, False, 7, 8),  # server_max_window_bits < 8
            (False, True, 8, 7),  # client_max_window_bits < 8
            (True, False, 16, 15),  # server_max_window_bits > 15
            (True, True, 15, 16),  # client_max_window_bits > 15
            (False, False, None, True),  # client_max_window_bits
            (False, False, True, None),  # server_max_window_bits
            (False, False, None, None, {"wbits": 11}),
            (False, False, None, None, {}, True),  # require_client_max_window_bits
        ]:
            with self.subTest(config=config):
                with self.assertRaises(ValueError):
                    ServerPerMessageDeflateFactory(*config)

    def test_process_request_params(self):
        # Parameters in result appear swapped vs. config because the order is
        # (remote, local) vs. (server, client).
        for config, request_params, response_params, result in [
            # Test without any parameter
            (
                (False, False, None, None),
                [],
                [],
                (False, False, 15, 15),
            ),
            (
                (False, False, None, None),
                [("unknown", None)],
                None,
                InvalidParameterName,
            ),
            # Test server_no_context_takeover
            (
                (False, False, None, None),
                [("server_no_context_takeover", None)],
                [("server_no_context_takeover", None)],
                (False, True, 15, 15),
            ),
            (
                (True, False, None, None),
                [],
                [("server_no_context_takeover", None)],
                (False, True, 15, 15),
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", None)],
                [("server_no_context_takeover", None)],
                (False, True, 15, 15),
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", None)] * 2,
                None,
                DuplicateParameter,
            ),
            (
                (True, False, None, None),
                [("server_no_context_takeover", "42")],
                None,
                InvalidParameterValue,
            ),
            # Test client_no_context_takeover
            (
                (False, False, None, None),
                [("client_no_context_takeover", None)],
                [("client_no_context_takeover", None)],  # doesn't matter
                (True, False, 15, 15),
            ),
            (
                (False, True, None, None),
                [],
                [("client_no_context_takeover", None)],
                (True, False, 15, 15),
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", None)],
                [("client_no_context_takeover", None)],  # doesn't matter
                (True, False, 15, 15),
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", None)] * 2,
                None,
                DuplicateParameter,
            ),
            (
                (False, True, None, None),
                [("client_no_context_takeover", "42")],
                None,
                InvalidParameterValue,
            ),
            # Test server_max_window_bits
            (
                (False, False, None, None),
                [("server_max_window_bits", "7")],
                None,
                NegotiationError,
            ),
            (
                (False, False, None, None),
                [("server_max_window_bits", "10")],
                [("server_max_window_bits", "10")],
                (False, False, 15, 10),
            ),
            (
                (False, False, None, None),
                [("server_max_window_bits", "16")],
                None,
                NegotiationError,
            ),
            (
                (False, False, 12, None),
                [],
                [("server_max_window_bits", "12")],
                (False, False, 15, 12),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "10")],
                [("server_max_window_bits", "10")],
                (False, False, 15, 10),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "12")],
                [("server_max_window_bits", "12")],
                (False, False, 15, 12),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "13")],
                [("server_max_window_bits", "12")],
                (False, False, 15, 12),
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "12")] * 2,
                None,
                DuplicateParameter,
            ),
            (
                (False, False, 12, None),
                [("server_max_window_bits", "42")],
                None,
                InvalidParameterValue,
            ),
            # Test client_max_window_bits
            (
                (False, False, None, None),
                [("client_max_window_bits", None)],
                [],
                (False, False, 15, 15),
            ),
            (
                (False, False, None, None),
                [("client_max_window_bits", "7")],
                None,
                InvalidParameterValue,
            ),
            (
                (False, False, None, None),
                [("client_max_window_bits", "10")],
                [("client_max_window_bits", "10")],  # doesn't matter
                (False, False, 10, 15),
            ),
            (
                (False, False, None, None),
                [("client_max_window_bits", "16")],
                None,
                InvalidParameterValue,
            ),
            (
                (False, False, None, 12),
                [],
                [],
                (False, False, 15, 15),
            ),
            (
                (False, False, None, 12, {}, True),
                [],
                None,
                NegotiationError,
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", None)],
                [("client_max_window_bits", "12")],
                (False, False, 12, 15),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "10")],
                [("client_max_window_bits", "10")],
                (False, False, 10, 15),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "12")],
                [("client_max_window_bits", "12")],  # doesn't matter
                (False, False, 12, 15),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "13")],
                [("client_max_window_bits", "12")],  # doesn't matter
                (False, False, 12, 15),
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "12")] * 2,
                None,
                DuplicateParameter,
            ),
            (
                (False, False, None, 12),
                [("client_max_window_bits", "42")],
                None,
                InvalidParameterValue,
            ),
            # Test all parameters together
            (
                (True, True, 12, 12),
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                (True, True, 10, 10),
            ),
            (
                (False, False, None, None),
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "10"),
                    ("client_max_window_bits", "10"),
                ],
                (True, True, 10, 10),
            ),
            (
                (True, True, 12, 12),
                [("client_max_window_bits", None)],
                [
                    ("server_no_context_takeover", None),
                    ("client_no_context_takeover", None),
                    ("server_max_window_bits", "12"),
                    ("client_max_window_bits", "12"),
                ],
                (True, True, 12, 12),
            ),
        ]:
            with self.subTest(
                config=config,
                request_params=request_params,
                response_params=response_params,
            ):
                factory = ServerPerMessageDeflateFactory(*config)
                if isinstance(result, type) and issubclass(result, Exception):
                    with self.assertRaises(result):
                        factory.process_request_params(request_params, [])
                else:
                    params, extension = factory.process_request_params(
                        request_params, []
                    )
                    self.assertEqual(params, response_params)
                    expected = PerMessageDeflate(*result)
                    self.assertExtensionEqual(extension, expected)

    def test_process_response_params_deduplication(self):
        factory = ServerPerMessageDeflateFactory(False, False, None, None)
        with self.assertRaises(NegotiationError):
            factory.process_request_params(
                [], [PerMessageDeflate(False, False, 15, 15)]
            )

    def test_enable_server_permessage_deflate(self):
        for extensions, (
            expected_len,
            expected_position,
            expected_compress_settings,
        ) in [
            (
                None,
                (1, 0, {"memLevel": 5}),
            ),
            (
                [],
                (1, 0, {"memLevel": 5}),
            ),
            (
                [ServerNoOpExtensionFactory()],
                (2, 1, {"memLevel": 5}),
            ),
            (
                [ServerPerMessageDeflateFactory(compress_settings={"memLevel": 7})],
                (1, 0, {"memLevel": 7}),
            ),
            (
                [
                    ServerPerMessageDeflateFactory(compress_settings={"memLevel": 7}),
                    ServerNoOpExtensionFactory(),
                ],
                (2, 0, {"memLevel": 7}),
            ),
            (
                [
                    ServerNoOpExtensionFactory(),
                    ServerPerMessageDeflateFactory(compress_settings={"memLevel": 7}),
                ],
                (2, 1, {"memLevel": 7}),
            ),
        ]:
            with self.subTest(extensions=extensions):
                extensions = enable_server_permessage_deflate(extensions)
                self.assertEqual(len(extensions), expected_len)
                extension = extensions[expected_position]
                self.assertIsInstance(extension, ServerPerMessageDeflateFactory)
                self.assertEqual(
                    extension.compress_settings,
                    expected_compress_settings,
                )
