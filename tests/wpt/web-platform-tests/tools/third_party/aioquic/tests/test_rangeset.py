from unittest import TestCase

from aioquic.quic.rangeset import RangeSet


class RangeSetTest(TestCase):
    def test_add_single_duplicate(self):
        rangeset = RangeSet()

        rangeset.add(0)
        self.assertEqual(list(rangeset), [range(0, 1)])

        rangeset.add(0)
        self.assertEqual(list(rangeset), [range(0, 1)])

    def test_add_single_ordered(self):
        rangeset = RangeSet()

        rangeset.add(0)
        self.assertEqual(list(rangeset), [range(0, 1)])

        rangeset.add(1)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(2)
        self.assertEqual(list(rangeset), [range(0, 3)])

    def test_add_single_merge(self):
        rangeset = RangeSet()

        rangeset.add(0)
        self.assertEqual(list(rangeset), [range(0, 1)])

        rangeset.add(2)
        self.assertEqual(list(rangeset), [range(0, 1), range(2, 3)])

        rangeset.add(1)
        self.assertEqual(list(rangeset), [range(0, 3)])

    def test_add_single_reverse(self):
        rangeset = RangeSet()

        rangeset.add(2)
        self.assertEqual(list(rangeset), [range(2, 3)])

        rangeset.add(1)
        self.assertEqual(list(rangeset), [range(1, 3)])

        rangeset.add(0)
        self.assertEqual(list(rangeset), [range(0, 3)])

    def test_add_range_ordered(self):
        rangeset = RangeSet()

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(2, 4)
        self.assertEqual(list(rangeset), [range(0, 4)])

        rangeset.add(4, 6)
        self.assertEqual(list(rangeset), [range(0, 6)])

    def test_add_range_merge(self):
        rangeset = RangeSet()

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(3, 5)
        self.assertEqual(list(rangeset), [range(0, 2), range(3, 5)])

        rangeset.add(2, 3)
        self.assertEqual(list(rangeset), [range(0, 5)])

    def test_add_range_overlap(self):
        rangeset = RangeSet()

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(3, 5)
        self.assertEqual(list(rangeset), [range(0, 2), range(3, 5)])

        rangeset.add(1, 5)
        self.assertEqual(list(rangeset), [range(0, 5)])

    def test_add_range_overlap_2(self):
        rangeset = RangeSet()

        rangeset.add(2, 4)
        rangeset.add(6, 8)
        rangeset.add(10, 12)
        rangeset.add(16, 18)
        self.assertEqual(
            list(rangeset), [range(2, 4), range(6, 8), range(10, 12), range(16, 18)]
        )

        rangeset.add(1, 15)
        self.assertEqual(list(rangeset), [range(1, 15), range(16, 18)])

    def test_add_range_reverse(self):
        rangeset = RangeSet()

        rangeset.add(6, 8)
        self.assertEqual(list(rangeset), [range(6, 8)])

        rangeset.add(3, 5)
        self.assertEqual(list(rangeset), [range(3, 5), range(6, 8)])

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2), range(3, 5), range(6, 8)])

    def test_add_range_unordered_contiguous(self):
        rangeset = RangeSet()

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(4, 6)
        self.assertEqual(list(rangeset), [range(0, 2), range(4, 6)])

        rangeset.add(2, 4)
        self.assertEqual(list(rangeset), [range(0, 6)])

    def test_add_range_unordered_sparse(self):
        rangeset = RangeSet()

        rangeset.add(0, 2)
        self.assertEqual(list(rangeset), [range(0, 2)])

        rangeset.add(6, 8)
        self.assertEqual(list(rangeset), [range(0, 2), range(6, 8)])

        rangeset.add(3, 5)
        self.assertEqual(list(rangeset), [range(0, 2), range(3, 5), range(6, 8)])

    def test_subtract(self):
        rangeset = RangeSet()
        rangeset.add(0, 10)
        rangeset.add(20, 30)

        rangeset.subtract(0, 3)
        self.assertEqual(list(rangeset), [range(3, 10), range(20, 30)])

    def test_subtract_no_change(self):
        rangeset = RangeSet()
        rangeset.add(5, 10)
        rangeset.add(15, 20)
        rangeset.add(25, 30)

        rangeset.subtract(0, 5)
        self.assertEqual(list(rangeset), [range(5, 10), range(15, 20), range(25, 30)])

        rangeset.subtract(10, 15)
        self.assertEqual(list(rangeset), [range(5, 10), range(15, 20), range(25, 30)])

    def test_subtract_overlap(self):
        rangeset = RangeSet()
        rangeset.add(1, 4)
        rangeset.add(6, 8)
        rangeset.add(10, 20)
        rangeset.add(30, 40)
        self.assertEqual(
            list(rangeset), [range(1, 4), range(6, 8), range(10, 20), range(30, 40)]
        )

        rangeset.subtract(0, 2)
        self.assertEqual(
            list(rangeset), [range(2, 4), range(6, 8), range(10, 20), range(30, 40)]
        )

        rangeset.subtract(3, 11)
        self.assertEqual(list(rangeset), [range(2, 3), range(11, 20), range(30, 40)])

    def test_subtract_split(self):
        rangeset = RangeSet()
        rangeset.add(0, 10)
        rangeset.subtract(2, 5)
        self.assertEqual(list(rangeset), [range(0, 2), range(5, 10)])

    def test_bool(self):
        with self.assertRaises(NotImplementedError):
            bool(RangeSet())

    def test_contains(self):
        rangeset = RangeSet()
        self.assertFalse(0 in rangeset)

        rangeset = RangeSet([range(0, 1)])
        self.assertTrue(0 in rangeset)
        self.assertFalse(1 in rangeset)

        rangeset = RangeSet([range(0, 1), range(3, 6)])
        self.assertTrue(0 in rangeset)
        self.assertFalse(1 in rangeset)
        self.assertFalse(2 in rangeset)
        self.assertTrue(3 in rangeset)
        self.assertTrue(4 in rangeset)
        self.assertTrue(5 in rangeset)
        self.assertFalse(6 in rangeset)

    def test_eq(self):
        r0 = RangeSet([range(0, 1)])
        r1 = RangeSet([range(1, 2), range(3, 4)])
        r2 = RangeSet([range(3, 4), range(1, 2)])

        self.assertTrue(r0 == r0)
        self.assertFalse(r0 == r1)
        self.assertFalse(r0 == 0)

        self.assertTrue(r1 == r1)
        self.assertFalse(r1 == r0)
        self.assertTrue(r1 == r2)
        self.assertFalse(r1 == 0)

        self.assertTrue(r2 == r2)
        self.assertTrue(r2 == r1)
        self.assertFalse(r2 == r0)
        self.assertFalse(r2 == 0)

    def test_len(self):
        rangeset = RangeSet()
        self.assertEqual(len(rangeset), 0)

        rangeset = RangeSet([range(0, 1)])
        self.assertEqual(len(rangeset), 1)

    def test_pop(self):
        rangeset = RangeSet([range(1, 2), range(3, 4)])
        r = rangeset.shift()
        self.assertEqual(r, range(1, 2))
        self.assertEqual(list(rangeset), [range(3, 4)])

    def test_repr(self):
        rangeset = RangeSet([range(1, 2), range(3, 4)])
        self.assertEqual(repr(rangeset), "RangeSet([range(1, 2), range(3, 4)])")
