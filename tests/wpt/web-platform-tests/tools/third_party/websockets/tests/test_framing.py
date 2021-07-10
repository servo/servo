import asyncio
import codecs
import unittest
import unittest.mock

from websockets.exceptions import PayloadTooBig, ProtocolError
from websockets.framing import *

from .utils import AsyncioTestCase


class FramingTests(AsyncioTestCase):
    def decode(self, message, mask=False, max_size=None, extensions=None):
        self.stream = asyncio.StreamReader(loop=self.loop)
        self.stream.feed_data(message)
        self.stream.feed_eof()
        frame = self.loop.run_until_complete(
            Frame.read(
                self.stream.readexactly,
                mask=mask,
                max_size=max_size,
                extensions=extensions,
            )
        )
        # Make sure all the data was consumed.
        self.assertTrue(self.stream.at_eof())
        return frame

    def encode(self, frame, mask=False, extensions=None):
        write = unittest.mock.Mock()
        frame.write(write, mask=mask, extensions=extensions)
        # Ensure the entire frame is sent with a single call to write().
        # Multiple calls cause TCP fragmentation and degrade performance.
        self.assertEqual(write.call_count, 1)
        # The frame data is the single positional argument of that call.
        self.assertEqual(len(write.call_args[0]), 1)
        self.assertEqual(len(write.call_args[1]), 0)
        return write.call_args[0][0]

    def round_trip(self, message, expected, mask=False, extensions=None):
        decoded = self.decode(message, mask, extensions=extensions)
        self.assertEqual(decoded, expected)
        encoded = self.encode(decoded, mask, extensions=extensions)
        if mask:  # non-deterministic encoding
            decoded = self.decode(encoded, mask, extensions=extensions)
            self.assertEqual(decoded, expected)
        else:  # deterministic encoding
            self.assertEqual(encoded, message)

    def round_trip_close(self, data, code, reason):
        parsed = parse_close(data)
        self.assertEqual(parsed, (code, reason))
        serialized = serialize_close(code, reason)
        self.assertEqual(serialized, data)

    def test_text(self):
        self.round_trip(b"\x81\x04Spam", Frame(True, OP_TEXT, b"Spam"))

    def test_text_masked(self):
        self.round_trip(
            b"\x81\x84\x5b\xfb\xe1\xa8\x08\x8b\x80\xc5",
            Frame(True, OP_TEXT, b"Spam"),
            mask=True,
        )

    def test_binary(self):
        self.round_trip(b"\x82\x04Eggs", Frame(True, OP_BINARY, b"Eggs"))

    def test_binary_masked(self):
        self.round_trip(
            b"\x82\x84\x53\xcd\xe2\x89\x16\xaa\x85\xfa",
            Frame(True, OP_BINARY, b"Eggs"),
            mask=True,
        )

    def test_non_ascii_text(self):
        self.round_trip(
            b"\x81\x05caf\xc3\xa9", Frame(True, OP_TEXT, "café".encode("utf-8"))
        )

    def test_non_ascii_text_masked(self):
        self.round_trip(
            b"\x81\x85\x64\xbe\xee\x7e\x07\xdf\x88\xbd\xcd",
            Frame(True, OP_TEXT, "café".encode("utf-8")),
            mask=True,
        )

    def test_close(self):
        self.round_trip(b"\x88\x00", Frame(True, OP_CLOSE, b""))

    def test_ping(self):
        self.round_trip(b"\x89\x04ping", Frame(True, OP_PING, b"ping"))

    def test_pong(self):
        self.round_trip(b"\x8a\x04pong", Frame(True, OP_PONG, b"pong"))

    def test_long(self):
        self.round_trip(
            b"\x82\x7e\x00\x7e" + 126 * b"a", Frame(True, OP_BINARY, 126 * b"a")
        )

    def test_very_long(self):
        self.round_trip(
            b"\x82\x7f\x00\x00\x00\x00\x00\x01\x00\x00" + 65536 * b"a",
            Frame(True, OP_BINARY, 65536 * b"a"),
        )

    def test_payload_too_big(self):
        with self.assertRaises(PayloadTooBig):
            self.decode(b"\x82\x7e\x04\x01" + 1025 * b"a", max_size=1024)

    def test_bad_reserved_bits(self):
        for encoded in [b"\xc0\x00", b"\xa0\x00", b"\x90\x00"]:
            with self.subTest(encoded=encoded):
                with self.assertRaises(ProtocolError):
                    self.decode(encoded)

    def test_good_opcode(self):
        for opcode in list(range(0x00, 0x03)) + list(range(0x08, 0x0B)):
            encoded = bytes([0x80 | opcode, 0])
            with self.subTest(encoded=encoded):
                self.decode(encoded)  # does not raise an exception

    def test_bad_opcode(self):
        for opcode in list(range(0x03, 0x08)) + list(range(0x0B, 0x10)):
            encoded = bytes([0x80 | opcode, 0])
            with self.subTest(encoded=encoded):
                with self.assertRaises(ProtocolError):
                    self.decode(encoded)

    def test_mask_flag(self):
        # Mask flag correctly set.
        self.decode(b"\x80\x80\x00\x00\x00\x00", mask=True)
        # Mask flag incorrectly unset.
        with self.assertRaises(ProtocolError):
            self.decode(b"\x80\x80\x00\x00\x00\x00")
        # Mask flag correctly unset.
        self.decode(b"\x80\x00")
        # Mask flag incorrectly set.
        with self.assertRaises(ProtocolError):
            self.decode(b"\x80\x00", mask=True)

    def test_control_frame_max_length(self):
        # At maximum allowed length.
        self.decode(b"\x88\x7e\x00\x7d" + 125 * b"a")
        # Above maximum allowed length.
        with self.assertRaises(ProtocolError):
            self.decode(b"\x88\x7e\x00\x7e" + 126 * b"a")

    def test_prepare_data_str(self):
        self.assertEqual(prepare_data("café"), (OP_TEXT, b"caf\xc3\xa9"))

    def test_prepare_data_bytes(self):
        self.assertEqual(prepare_data(b"tea"), (OP_BINARY, b"tea"))

    def test_prepare_data_bytearray(self):
        self.assertEqual(
            prepare_data(bytearray(b"tea")), (OP_BINARY, bytearray(b"tea"))
        )

    def test_prepare_data_memoryview(self):
        self.assertEqual(
            prepare_data(memoryview(b"tea")), (OP_BINARY, memoryview(b"tea"))
        )

    def test_prepare_data_non_contiguous_memoryview(self):
        self.assertEqual(prepare_data(memoryview(b"tteeaa")[::2]), (OP_BINARY, b"tea"))

    def test_prepare_data_list(self):
        with self.assertRaises(TypeError):
            prepare_data([])

    def test_prepare_data_none(self):
        with self.assertRaises(TypeError):
            prepare_data(None)

    def test_encode_data_str(self):
        self.assertEqual(encode_data("café"), b"caf\xc3\xa9")

    def test_encode_data_bytes(self):
        self.assertEqual(encode_data(b"tea"), b"tea")

    def test_encode_data_bytearray(self):
        self.assertEqual(encode_data(bytearray(b"tea")), b"tea")

    def test_encode_data_memoryview(self):
        self.assertEqual(encode_data(memoryview(b"tea")), b"tea")

    def test_encode_data_non_contiguous_memoryview(self):
        self.assertEqual(encode_data(memoryview(b"tteeaa")[::2]), b"tea")

    def test_encode_data_list(self):
        with self.assertRaises(TypeError):
            encode_data([])

    def test_encode_data_none(self):
        with self.assertRaises(TypeError):
            encode_data(None)

    def test_fragmented_control_frame(self):
        # Fin bit correctly set.
        self.decode(b"\x88\x00")
        # Fin bit incorrectly unset.
        with self.assertRaises(ProtocolError):
            self.decode(b"\x08\x00")

    def test_parse_close_and_serialize_close(self):
        self.round_trip_close(b"\x03\xe8", 1000, "")
        self.round_trip_close(b"\x03\xe8OK", 1000, "OK")

    def test_parse_close_empty(self):
        self.assertEqual(parse_close(b""), (1005, ""))

    def test_parse_close_errors(self):
        with self.assertRaises(ProtocolError):
            parse_close(b"\x03")
        with self.assertRaises(ProtocolError):
            parse_close(b"\x03\xe7")
        with self.assertRaises(UnicodeDecodeError):
            parse_close(b"\x03\xe8\xff\xff")

    def test_serialize_close_errors(self):
        with self.assertRaises(ProtocolError):
            serialize_close(999, "")

    def test_extensions(self):
        class Rot13:
            @staticmethod
            def encode(frame):
                assert frame.opcode == OP_TEXT
                text = frame.data.decode()
                data = codecs.encode(text, "rot13").encode()
                return frame._replace(data=data)

            # This extensions is symmetrical.
            @staticmethod
            def decode(frame, *, max_size=None):
                return Rot13.encode(frame)

        self.round_trip(
            b"\x81\x05uryyb", Frame(True, OP_TEXT, b"hello"), extensions=[Rot13()]
        )
