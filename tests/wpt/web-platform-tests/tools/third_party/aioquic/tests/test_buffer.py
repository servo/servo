from unittest import TestCase

from aioquic.buffer import Buffer, BufferReadError, BufferWriteError, size_uint_var


class BufferTest(TestCase):
    def test_data_slice(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.data_slice(0, 8), b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.data_slice(1, 3), b"\x07\x06")

        with self.assertRaises(BufferReadError):
            buf.data_slice(-1, 3)
        with self.assertRaises(BufferReadError):
            buf.data_slice(0, 9)
        with self.assertRaises(BufferReadError):
            buf.data_slice(1, 0)

    def test_pull_bytes(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_bytes(3), b"\x08\x07\x06")

    def test_pull_bytes_negative(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        with self.assertRaises(BufferReadError):
            buf.pull_bytes(-1)

    def test_pull_bytes_truncated(self):
        buf = Buffer(capacity=0)
        with self.assertRaises(BufferReadError):
            buf.pull_bytes(2)
        self.assertEqual(buf.tell(), 0)

    def test_pull_bytes_zero(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_bytes(0), b"")

    def test_pull_uint8(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_uint8(), 0x08)
        self.assertEqual(buf.tell(), 1)

    def test_pull_uint8_truncated(self):
        buf = Buffer(capacity=0)
        with self.assertRaises(BufferReadError):
            buf.pull_uint8()
        self.assertEqual(buf.tell(), 0)

    def test_pull_uint16(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_uint16(), 0x0807)
        self.assertEqual(buf.tell(), 2)

    def test_pull_uint16_truncated(self):
        buf = Buffer(capacity=1)
        with self.assertRaises(BufferReadError):
            buf.pull_uint16()
        self.assertEqual(buf.tell(), 0)

    def test_pull_uint32(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_uint32(), 0x08070605)
        self.assertEqual(buf.tell(), 4)

    def test_pull_uint32_truncated(self):
        buf = Buffer(capacity=3)
        with self.assertRaises(BufferReadError):
            buf.pull_uint32()
        self.assertEqual(buf.tell(), 0)

    def test_pull_uint64(self):
        buf = Buffer(data=b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.pull_uint64(), 0x0807060504030201)
        self.assertEqual(buf.tell(), 8)

    def test_pull_uint64_truncated(self):
        buf = Buffer(capacity=7)
        with self.assertRaises(BufferReadError):
            buf.pull_uint64()
        self.assertEqual(buf.tell(), 0)

    def test_push_bytes(self):
        buf = Buffer(capacity=3)
        buf.push_bytes(b"\x08\x07\x06")
        self.assertEqual(buf.data, b"\x08\x07\x06")
        self.assertEqual(buf.tell(), 3)

    def test_push_bytes_truncated(self):
        buf = Buffer(capacity=3)
        with self.assertRaises(BufferWriteError):
            buf.push_bytes(b"\x08\x07\x06\x05")
        self.assertEqual(buf.tell(), 0)

    def test_push_bytes_zero(self):
        buf = Buffer(capacity=3)
        buf.push_bytes(b"")
        self.assertEqual(buf.data, b"")
        self.assertEqual(buf.tell(), 0)

    def test_push_uint8(self):
        buf = Buffer(capacity=1)
        buf.push_uint8(0x08)
        self.assertEqual(buf.data, b"\x08")
        self.assertEqual(buf.tell(), 1)

    def test_push_uint16(self):
        buf = Buffer(capacity=2)
        buf.push_uint16(0x0807)
        self.assertEqual(buf.data, b"\x08\x07")
        self.assertEqual(buf.tell(), 2)

    def test_push_uint32(self):
        buf = Buffer(capacity=4)
        buf.push_uint32(0x08070605)
        self.assertEqual(buf.data, b"\x08\x07\x06\x05")
        self.assertEqual(buf.tell(), 4)

    def test_push_uint64(self):
        buf = Buffer(capacity=8)
        buf.push_uint64(0x0807060504030201)
        self.assertEqual(buf.data, b"\x08\x07\x06\x05\x04\x03\x02\x01")
        self.assertEqual(buf.tell(), 8)

    def test_seek(self):
        buf = Buffer(data=b"01234567")
        self.assertFalse(buf.eof())
        self.assertEqual(buf.tell(), 0)

        buf.seek(4)
        self.assertFalse(buf.eof())
        self.assertEqual(buf.tell(), 4)

        buf.seek(8)
        self.assertTrue(buf.eof())
        self.assertEqual(buf.tell(), 8)

        with self.assertRaises(BufferReadError):
            buf.seek(-1)
        self.assertEqual(buf.tell(), 8)
        with self.assertRaises(BufferReadError):
            buf.seek(9)
        self.assertEqual(buf.tell(), 8)


class UintVarTest(TestCase):
    def roundtrip(self, data, value):
        buf = Buffer(data=data)
        self.assertEqual(buf.pull_uint_var(), value)
        self.assertEqual(buf.tell(), len(data))

        buf = Buffer(capacity=8)
        buf.push_uint_var(value)
        self.assertEqual(buf.data, data)

    def test_uint_var(self):
        # 1 byte
        self.roundtrip(b"\x00", 0)
        self.roundtrip(b"\x01", 1)
        self.roundtrip(b"\x25", 37)
        self.roundtrip(b"\x3f", 63)

        # 2 bytes
        self.roundtrip(b"\x7b\xbd", 15293)
        self.roundtrip(b"\x7f\xff", 16383)

        # 4 bytes
        self.roundtrip(b"\x9d\x7f\x3e\x7d", 494878333)
        self.roundtrip(b"\xbf\xff\xff\xff", 1073741823)

        # 8 bytes
        self.roundtrip(b"\xc2\x19\x7c\x5e\xff\x14\xe8\x8c", 151288809941952652)
        self.roundtrip(b"\xff\xff\xff\xff\xff\xff\xff\xff", 4611686018427387903)

    def test_pull_uint_var_truncated(self):
        buf = Buffer(capacity=0)
        with self.assertRaises(BufferReadError):
            buf.pull_uint_var()

        buf = Buffer(data=b"\xff")
        with self.assertRaises(BufferReadError):
            buf.pull_uint_var()

    def test_push_uint_var_too_big(self):
        buf = Buffer(capacity=8)
        with self.assertRaises(ValueError) as cm:
            buf.push_uint_var(4611686018427387904)
        self.assertEqual(
            str(cm.exception), "Integer is too big for a variable-length integer"
        )

    def test_size_uint_var(self):
        self.assertEqual(size_uint_var(63), 1)
        self.assertEqual(size_uint_var(16383), 2)
        self.assertEqual(size_uint_var(1073741823), 4)
        self.assertEqual(size_uint_var(4611686018427387903), 8)

        with self.assertRaises(ValueError) as cm:
            size_uint_var(4611686018427387904)
        self.assertEqual(
            str(cm.exception), "Integer is too big for a variable-length integer"
        )
