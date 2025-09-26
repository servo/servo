import json
import os
import re
import sys
from typing import IO, Container, Dict, Iterable, Optional


GIT_PUSH = re.compile(
    r"^(?P<flag>.)\t(?P<from_ref>[^\t:]*):(?P<to_ref>[^\t:]*)\t(?P<summary>.*?)(?: \((?P<reason>.*)\))?\n$"
)


def parse_push(fd: IO[str]) -> Iterable[Dict[str, Optional[str]]]:
    for line in fd:
        m = GIT_PUSH.match(line)
        if m is not None:
            yield m.groupdict()


def process_push(fd: IO[str], refs: Container[str]) -> Dict[str, Optional[str]]:
    updated_refs = {}

    for ref_status in parse_push(fd):
        flag = ref_status["flag"]
        if flag not in (" ", "+", "-", "*"):
            continue

        to_ref = ref_status["to_ref"]
        summary = ref_status["summary"]
        assert to_ref is not None
        assert summary is not None

        if to_ref in refs:
            sha = None

            if flag in (" ", "+"):
                if "..." in summary:
                    _, sha = summary.split("...", maxsplit=1)
                elif ".." in summary:
                    _, sha = summary.split("..", maxsplit=1)

            updated_refs[to_ref] = sha

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
