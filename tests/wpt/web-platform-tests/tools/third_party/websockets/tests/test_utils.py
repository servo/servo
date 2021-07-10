import itertools
import unittest

from websockets.utils import apply_mask as py_apply_mask


class UtilsTests(unittest.TestCase):
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
        for data_type, mask_type in self.apply_mask_type_combos:
            for data_in, mask, data_out in self.apply_mask_test_values:
                data_in, mask = data_type(data_in), mask_type(mask)
                data_in, mask = memoryview(data_in), memoryview(mask)

                with self.subTest(data_in=data_in, mask=mask):
                    result = self.apply_mask(data_in, mask)
                    self.assertEqual(result, data_out)

    def test_apply_mask_non_contiguous_memoryview(self):
        for data_type, mask_type in self.apply_mask_type_combos:
            for data_in, mask, data_out in self.apply_mask_test_values:
                data_in, mask = data_type(data_in), mask_type(mask)
                data_in, mask = memoryview(data_in), memoryview(mask)
                data_in, mask = data_in[::-1], mask[::-1]
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
except ImportError:  # pragma: no cover
    pass
else:

    class SpeedupsTests(UtilsTests):
        @staticmethod
        def apply_mask(*args, **kwargs):
            return c_apply_mask(*args, **kwargs)

        def test_apply_mask_non_contiguous_memoryview(self):
            for data_type, mask_type in self.apply_mask_type_combos:
                for data_in, mask, data_out in self.apply_mask_test_values:
                    data_in, mask = data_type(data_in), mask_type(mask)
                    data_in, mask = memoryview(data_in), memoryview(mask)
                    data_in, mask = data_in[::-1], mask[::-1]
                    data_out = data_out[::-1]

                    with self.subTest(data_in=data_in, mask=mask):
                        # The C extension only supports contiguous memoryviews.
                        with self.assertRaises(TypeError):
                            self.apply_mask(data_in, mask)
