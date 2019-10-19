from tools.wpt import revlist


def test_calculate_cutoff_date():
    assert revlist.calculate_cutoff_date(3601, 3600, 0) == 3600
    assert revlist.calculate_cutoff_date(3600, 3600, 0) == 3600
    assert revlist.calculate_cutoff_date(3599, 3600, 0) == 0
    assert revlist.calculate_cutoff_date(3600, 3600, 1) == 1
    assert revlist.calculate_cutoff_date(3600, 3600, -1) == 3599


def test_parse_epoch():
    assert revlist.parse_epoch(b"10h") == 36000
    assert revlist.parse_epoch(b"10d") == 864000
    assert revlist.parse_epoch(b"10w") == 6048000
