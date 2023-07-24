# mypy: allow-untyped-defs

from unittest import mock

from tools.wpt import revlist


def test_calculate_cutoff_date():
    assert revlist.calculate_cutoff_date(3601, 3600, 0) == 3600
    assert revlist.calculate_cutoff_date(3600, 3600, 0) == 3600
    assert revlist.calculate_cutoff_date(3599, 3600, 0) == 0
    assert revlist.calculate_cutoff_date(3600, 3600, 1) == 1
    assert revlist.calculate_cutoff_date(3600, 3600, -1) == 3599


def test_parse_epoch():
    assert revlist.parse_epoch("10h") == 36000
    assert revlist.parse_epoch("10d") == 864000
    assert revlist.parse_epoch("10w") == 6048000

def check_revisions(tagged_revisions, expected_revisions):
    for tagged, expected in zip(tagged_revisions, expected_revisions):
        assert tagged == expected

@mock.patch('subprocess.check_output')
def test_get_epoch_revisions(mocked_check_output):
    # check:
    #
    # * Several revisions in the same epoch offset (BC, DEF, HIJ, and LM)
    # * Revision with a timestamp exactly equal to the epoch boundary (H)
    # * Revision in non closed interval (O)
    #
    #       mon  tue   wed   thu   fri   sat   sun   mon   thu   wed
    #          |     |     |     |     |     |     |     |     |
    #       -A---B-C---DEF---G---H--IJ----------K-----L-M----N--O--
    #                                                             ^
    #                                                            until
    #       max_count: 5; epoch: 1d
    #       Expected result: N,M,K,J,G,F,C,A
    epoch = 86400
    until = 1188000  # Wednesday, 14 January 1970 18:00:00 UTC
    mocked_check_output.return_value = b'''
merge_pr_O O 1166400 _wed_
merge_pr_N N 1080000 _tue_
merge_pr_M M 1015200 _mon_
merge_pr_L L 993600 _mon_
merge_pr_K K 907200 _sun_
merge_pr_J J 734400 _fri_
merge_pr_I I 712800 _fri_
merge_pr_H H 691200 _fri_
merge_pr_G G 648000 _thu_
merge_pr_F F 583200 _wed_
merge_pr_E E 561600 _wed_
merge_pr_D D 540000 _wed_
merge_pr_C C 475200 _tue_
merge_pr_B B 453600 _tue_
merge_pr_A A 388800 _mon_
'''
    tagged_revisions = revlist.get_epoch_revisions(epoch, until, 8)
    check_revisions(tagged_revisions, ['N', 'M', 'K', 'J', 'G', 'F', 'C', 'A'])
    assert len(list(tagged_revisions)) == 0  # generator exhausted


    # check: max_count with enough candidate items in the revision list
    #
    #       mon  tue   wed   thu   fri   sat   sun    mon
    #          |     |     |     |     |     |     |
    #       ------B-----C-----D----E-----F-----G------H---
    #                                                   ^
    #                                                 until
    #       max_count: 5; epoch: 1d
    #       Expected result: G,F,E,D,C
    epoch = 86400
    until = 1015200   # Monday, 12 January 1970 18:00:00 UTC
    mocked_check_output.return_value = b'''
merge_pr_H H 993600 _mon_
merge_pr_G G 907200 _sun_
merge_pr_F F 820800 _sat_
merge_pr_E E 734400 _fri_
merge_pr_D D 648000 _thu_
merge_pr_C C 561600 _wed_
merge_pr_B B 475200 _thu_
'''
    tagged_revisions = revlist.get_epoch_revisions(epoch, until, 5)
    check_revisions(tagged_revisions, ['G', 'F', 'E', 'D', 'C'])
    assert len(list(tagged_revisions)) == 0  # generator exhausted


    # check: max_count with less returned candidates items than the needed
    #
    #       mon  tue   wed   thu   fri   sat   sun   mon
    #          |     |     |     |     |     |     |
    #       -----------------------------F-----G------H---
    #                                                   ^
    #                                                 until
    #       max_count: 5; epoch: 1d
    #       Expected result: G,F
    epoch = 86400
    until = 1015200   # Monday, 12 January 1970 18:00:00 UTC
    mocked_check_output.return_value = b'''
merge_pr_H H 993600 _mon_
merge_pr_G G 907200 _sun_
merge_pr_F F 820800 _sat_
'''
    tagged_revisions = revlist.get_epoch_revisions(epoch, until, 5)
    check_revisions(tagged_revisions, ['G', 'F'])
    assert len(list(tagged_revisions)) == 0  # generator exhausted


    # check: initial until value is on an epoch boundary
    #
    #                      sud  mon   tue   wed   thu
    #                         |     |     |     |
    #                      -F-G-----------------H
    #                                           ^
    #                                         until
    #       max_count: 3; epoch: 1d
    #       Expected result: G,F
    #       * H is skipped because because the epoch
    #         interval is defined as an right-open interval
    #       * G is included but in the Monday's interval
    #       * F is included because it is the unique candidate
    #         included in the Sunday's interval
    epoch = 86400
    until = 1296000  # Thursday, 15 January 1970 0:00:00 UTC
    mocked_check_output.return_value = b'''
merge_pr_H H 1296000 _wed_
merge_pr_G G 950400 _mon_
merge_pr_F F 921600 _sud_
'''
    tagged_revisions = revlist.get_epoch_revisions(epoch, until, 3)
    check_revisions(tagged_revisions, ['G', 'F'])
    assert len(list(tagged_revisions)) == 0  # generator exhausted


    # check: until aligned with Monday, 5 January 1970 0:00:00 (345600)
    #        not with Thursday, 1 January 1970 0:00:00 (0)
    #
    #                      sud  mon   tue   wed   thu
    #                         |     |     |     |
    #                      -F-G--------------H---
    #                                           ^
    #                                         until
    #       max_count: 1; epoch: 1w
    #       Expected result: F
    epoch = 604800
    moday = 950400  # Monday, 12 January 1970 00:00:00 UTC
    until = moday + 345600  # 1296000. Thursday, 15 January 1970 0:00:00 UTC
    mocked_check_output.return_value = b'''
merge_pr_H H 1180800 _wed_
merge_pr_G G 950400 _mon_
merge_pr_F F 921600 _sud_
'''
    tagged_revisions = revlist.get_epoch_revisions(epoch, until, 1)
    check_revisions(tagged_revisions, ['F'])
    assert len(list(tagged_revisions)) == 0  # generator exhausted
