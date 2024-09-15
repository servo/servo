import json
import os
import re
import sys
from typing import IO, Container, Dict, Iterable, List, Optional

GIT_PUSH = re.compile(
    r"^(?P<flag>.)\t(?P<from>[^\t:]*):(?P<to>[^\t:]*)\t(?P<summary>.*[^\)])(?: \((?P<reason>[^\)]*)\))?\n$"
)


def parse_push(fd: IO[str]) -> Iterable[Dict[str, Optional[str]]]:
    for line in fd:
        m = GIT_PUSH.match(line)
        if m is not None:
            yield m.groupdict()


def process_push(fd: IO[str], refs: Container[str]) -> List[str]:
    updated_refs = []

    for ref_status in parse_push(fd):
        flag = ref_status["flag"]
        if flag not in (" ", "+", "-", "*"):
            continue

        to = ref_status["to"]
        assert to is not None
        if to in refs:
            updated_refs.append(to)

    return updated_refs


def main() -> None:
    git_push_output = os.environ["GIT_PUSH_OUTPUT"]
    refs = json.loads(os.environ["REFS"])

    with open(git_push_output, "r") as fd:
        updated_refs = process_push(fd, refs)

    json.dump(updated_refs, sys.stdout, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
