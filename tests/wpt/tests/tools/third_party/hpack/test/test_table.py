# -*- coding: utf-8 -*-
import pytest

from hpack import InvalidTableIndex
from hpack.table import HeaderTable, table_entry_size


class TestPackageFunctions:
    def test_table_entry_size(self):
        res = table_entry_size(b'TestName', b'TestValue')
        assert res == 49


class TestHeaderTable:
    def test_get_by_index_dynamic_table(self):
        tbl = HeaderTable()
        off = len(HeaderTable.STATIC_TABLE)
        val = (b'TestName', b'TestValue')
        tbl.add(*val)
        res = tbl.get_by_index(off + 1)
        assert res == val

    def test_get_by_index_static_table(self):
        tbl = HeaderTable()
        exp = (b':authority', b'')
        res = tbl.get_by_index(1)
        assert res == exp
        idx = len(HeaderTable.STATIC_TABLE)
        exp = (b'www-authenticate', b'')
        res = tbl.get_by_index(idx)
        assert res == exp

    def test_get_by_index_zero_index(self):
        tbl = HeaderTable()
        with pytest.raises(InvalidTableIndex):
            tbl.get_by_index(0)

    def test_get_by_index_out_of_range(self):
        tbl = HeaderTable()
        off = len(HeaderTable.STATIC_TABLE)
        tbl.add(b'TestName', b'TestValue')
        with pytest.raises(InvalidTableIndex) as e:
            tbl.get_by_index(off + 2)

        assert (
            "Invalid table index %d" % (off + 2) in str(e.value)
        )

    def test_repr(self):
        tbl = HeaderTable()
        tbl.add(b'TestName1', b'TestValue1')
        tbl.add(b'TestName2', b'TestValue2')
        tbl.add(b'TestName2', b'TestValue2')
        exp = (
            "HeaderTable(4096, False, deque(["
            "(b'TestName2', b'TestValue2'), "
            "(b'TestName2', b'TestValue2'), "
            "(b'TestName1', b'TestValue1')"
            "]))"
        )
        res = repr(tbl)
        assert res == exp

    def test_add_to_large(self):
        tbl = HeaderTable()
        # Max size to small to hold the value we specify
        tbl.maxsize = 1
        tbl.add(b'TestName', b'TestValue')
        # Table length should be 0
        assert len(tbl.dynamic_entries) == 0

    def test_search_in_static_full(self):
        tbl = HeaderTable()
        itm = (b':authority', b'')
        exp = (1, itm[0], itm[1])
        res = tbl.search(itm[0], itm[1])
        assert res == exp

    def test_search_in_static_partial(self):
        tbl = HeaderTable()
        exp = (1, b':authority', None)
        res = tbl.search(b':authority', b'NotInTable')
        assert res == exp

    def test_search_in_dynamic_full(self):
        tbl = HeaderTable()
        idx = len(HeaderTable.STATIC_TABLE) + 1
        tbl.add(b'TestName', b'TestValue')
        exp = (idx, b'TestName', b'TestValue')
        res = tbl.search(b'TestName', b'TestValue')
        assert res == exp

    def test_search_in_dynamic_partial(self):
        tbl = HeaderTable()
        idx = len(HeaderTable.STATIC_TABLE) + 1
        tbl.add(b'TestName', b'TestValue')
        exp = (idx, b'TestName', None)
        res = tbl.search(b'TestName', b'NotInTable')
        assert res == exp

    def test_search_no_match(self):
        tbl = HeaderTable()
        tbl.add(b'TestName', b'TestValue')
        res = tbl.search(b'NotInTable', b'NotInTable')
        assert res is None

    def test_maxsize_prop_getter(self):
        tbl = HeaderTable()
        assert tbl.maxsize == HeaderTable.DEFAULT_SIZE

    def test_maxsize_prop_setter(self):
        tbl = HeaderTable()
        exp = int(HeaderTable.DEFAULT_SIZE / 2)
        tbl.maxsize = exp
        assert tbl.resized is True
        assert tbl.maxsize == exp
        tbl.resized = False
        tbl.maxsize = exp
        assert tbl.resized is False
        assert tbl.maxsize == exp

    def test_size(self):
        tbl = HeaderTable()
        for i in range(3):
            tbl.add(b'TestName', b'TestValue')
        res = tbl._current_size
        assert res == 147

    def test_shrink_maxsize_is_zero(self):
        tbl = HeaderTable()
        tbl.add(b'TestName', b'TestValue')
        assert len(tbl.dynamic_entries) == 1
        tbl.maxsize = 0
        assert len(tbl.dynamic_entries) == 0

    def test_shrink_maxsize(self):
        tbl = HeaderTable()
        for i in range(3):
            tbl.add(b'TestName', b'TestValue')

        assert tbl._current_size == 147
        tbl.maxsize = 146
        assert len(tbl.dynamic_entries) == 2
        assert tbl._current_size == 98
