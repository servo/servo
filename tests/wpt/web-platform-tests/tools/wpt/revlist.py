import argparse
import logging
import os
import time
from tools.wpt.testfiles import get_git_cmd

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

logger = logging.getLogger()

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Dict
    from typing import List
    from typing import Text


def calculate_cutoff_date(until, epoch, offset):
    return ((((until - offset) // epoch)) * epoch) + offset


def parse_epoch(string):
    # type: (str) -> int
    UNIT_DICT = {"h": 3600, "d": 86400, "w": 604800}
    base = string[:-1]
    unit = string[-1:]
    if base.isdigit() and unit in UNIT_DICT:
        return int(base) * UNIT_DICT[unit]
    raise argparse.ArgumentTypeError('must be digits followed by h/d/w')


def get_tagged_revisions(pattern):
    # type: (bytes) -> List[..., Dict]
    '''
    Returns the tagged revisions indexed by the committer date.
    '''
    git = get_git_cmd(wpt_root)
    args = [
        pattern,
        '--sort=-committerdate',
        '--format=%(refname:lstrip=2) %(objectname) %(committerdate:raw)',
        '--count=100000'
    ]
    for line in git("for-each-ref", *args).splitlines():
        tag, commit, date, _ = line.split(" ")
        date = int(date)
        yield tag, commit, date


def get_epoch_revisions(epoch, until, max_count):
    # type: (**Any) -> List[Text]
    logger.debug("get_epoch_revisions(%s, %s)" % (epoch, max_count))
    # Set an offset to start to count the the weekly epoch from
    # Monday 00:00:00. This is particularly important for the weekly epoch
    # because fix the start of the epoch to Monday. This offset is calculated
    # from Thursday, 1 January 1970 0:00:00 to Monday, 5 January 1970 0:00:00
    epoch_offset = 345600
    count = 0

    # Iterates the tagged revisions in descending order finding the more
    # recent commit still older than a "cutoff_date" value.
    # When a commit is found "cutoff_date" is set to a new value multiplier of
    # "epoch" but still below of the date of the current commit found.
    # This needed to deal with intervals where no candidates were found
    # for the current "epoch" and the next candidate found is yet below
    # the lower values of the interval (it is the case of J and I for the
    # interval between Wed and Tue, in the example). The algorithm fix
    # the next "cutoff_date" value based on the date value of the current one
    # skipping the intermediate values.
    # The loop ends once we reached the required number of revisions to return
    # or the are no more tagged revisions or the cutoff_date reach zero.
    #
    #   Fri   Sat   Sun   Mon   Tue   Wed   Thu   Fri   Sat
    #    |     |     |     |     |     |     |     |     |
    # -A---B-C---DEF---G---H--IJ----------K-----L-M----N--O--
    #                                                       ^
    #                                                      now
    # Expected result: N,M,K,J,H,G,F,C,A

    cutoff_date = calculate_cutoff_date(until, epoch, epoch_offset)
    for _, commit, date in get_tagged_revisions("refs/tags/merge_pr_*"):
        if count >= max_count:
            return
        if date < cutoff_date:
            yield commit
            count += 1
            cutoff_date = calculate_cutoff_date(date, epoch, epoch_offset)


def get_parser():
    # type: () -> argparse.ArgumentParser
    parser = argparse.ArgumentParser()
    parser.add_argument("--epoch",
                        default="1d",
                        type=parse_epoch,
                        help="regular interval of time selected to get the "
                             "tagged revisions. Valid values are digits "
                             "followed by h/d/w (e.x. 9h, 9d, 9w ...) where "
                             "the mimimun selectable interval is one hour "
                             "(1h)")
    parser.add_argument("--max-count",
                        default=1,
                        type=int,
                        help="maximum number of revisions to be returned by "
                             "the command")
    return parser


def run_rev_list(**kwargs):
    # type: (**Any) -> None
    # "epoch_threshold" is a safety margin. After this time it is fine to
    # assume that any tags are created and pushed.
    epoch_threshold = 600
    until = int(time.time()) - epoch_threshold
    for line in get_epoch_revisions(kwargs["epoch"], until, kwargs["max_count"]):
        print(line)
