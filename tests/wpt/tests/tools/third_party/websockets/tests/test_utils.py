import base64
import itertools
import platform
import unittest

from websockets.utils import accept_key, apply_mask as py_apply_mask, generate_key


# Test vector from RFC 6455
KEY = "dGhlIHNhbXBsZSBub25jZQ=="
ACCEPT = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="


class UtilsTests(unittest.TestCase):
    def test_generate_key(self):
        key = generate_key()
        self.assertEqual(len(base64.b64decode(key.encode())), 16)

    def test_accept_key(self):
        self.assertEqual(accept_key(KEY), ACCEPT)


class ApplyMaskTests(unittest.TestCase):
    @staticmethod
    def apply_mask(*args, **kwargs):
        return py_apply_mask(*args, **kwargs)

    apply_mask_type_combos = list(itertools.product([bytes, bytearray], repeat=2))

    apply_mask_test_values = [
        (b"", b"1234", b""),
        (b"aBcDe", b"\x00\x00\x00\x00", b"aBcDe"),
        (b"abcdABCD", b"1234", b"PPPPpppp"),
        (b"abcdABCD" * 10, b"1234", b"PPPPpppp" * 10),
    ]

    def test_apply_mask(self):
        for data_type, mask_type in self.apply_mask_type_combos:
            for data_in, mask, data_out in self.apply_mask_test_values:
                data_in, mask = data_type(data_in), mask_type(mask)

                with self.subTest(data_in=data_in, mask=mask):
                    result = self.apply_mask(data_in, mask)
                    self.assertEqual(result, data_out)

    def test_apply_mask_memoryview(self):
        for mask_type in [bytes, bytearray]:
            for data_in, mask, data_out in self.apply_mask_test_values:
                data_in, mask = memoryview(data_in), mask_type(mask)

                with self.subTest(data_in=data_in, mask=mask):
                    result = self.apply_mask(data_in, mask)
                    self.assertEqual(result, data_out)

    def test_apply_mask_non_contiguous_memoryview(self):
        for mask_type in [bytes, bytearray]:
            for data_in, mask, data_out in self.apply_mask_test_values:
                data_in, mask = memoryview(data_in)[::-1], mask_type(mask)[::-1]
                data_out = data_out[::-1]

                with self.subTest(data_in=data_in, mask=mask):
                    result = self.apply_mask(data_in, mask)
                    self.assertEqual(result, data_out)

    def test_apply_mask_check_input_types(self):
        for data_in, mask in [(None, None), (b"abcd", None), (None, b"abcd")]:
            with self.subTest(data_in=data_in, mask=mask):
                with self.assertRaises(TypeError):
                    self.apply_mask(data_in, mask)

    def test_apply_mask_check_mask_length(self):
        for data_in, mask in [
            (b"", b""),
            (b"abcd", b"123"),
            (b"", b"aBcDe"),
            (b"12345678", b"12345678"),
        ]:
            with self.subTest(data_in=data_in, mask=mask):
                with self.assertRaises(ValueError):
                    self.apply_mask(data_in, mask)


try:
    from websockets.speedups import apply_mask as c_apply_mask
except ImportError:
    pass
else:

    class SpeedupsTests(ApplyMaskTests):
        @staticmethod
        def apply_mask(*args, **kwargs):
            try:
                return c_apply_mask(*args, **kwargs)
            except NotImplementedError as exc:  # pragma: no cover
                # PyPy doesn't implement creating contiguous readonly buffer
                # from non-contiguous. We don't care about this edge case.
                if (
                    platform.python_implementation() == "PyPy"
                    and "not implemented yet" in str(exc)
                ):
                    raise unittest.SkipTest(str(exc))
                else:
                    raise
