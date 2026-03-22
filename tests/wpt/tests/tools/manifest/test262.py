from __future__ import print_function

from dataclasses import dataclass
from logging import Logger
from typing import Dict, List, Optional, Text, Tuple

import re

# Matches trailing whitespace and any following blank lines.
_BLANK_LINES = r"([ \t]*[\r\n]{1,2})*"

# Matches the YAML frontmatter block.
_YAML_PATTERN = re.compile(r"/\*---(.*)---\*/" + _BLANK_LINES, re.DOTALL)

_STRIP_CONTROL_CHARS = re.compile(r'[\x7f-\x9f]')



@dataclass
class TestRecord:
    test: str
    includes: Optional[List[Text]] = None
    negative: Optional[Dict[Text, Text]] = None
    is_only_strict: bool = False
    is_module: bool = False

def _yaml_attr_parser(logger: Logger, test_record: TestRecord, attrs: Text, name: Text) -> None:
    import yaml
    parsed = yaml.safe_load(re.sub(_STRIP_CONTROL_CHARS, ' ', attrs))
    if parsed is None:
        logger.error("Failed to parse yaml in name %s" % name)
        return

    for key, value in parsed.items():
        if key == "negative":
            test_record.negative = value
        elif key == "flags":
            if isinstance(value, list):
                for flag in value:
                    if flag == "onlyStrict":
                        test_record.is_only_strict = True
                    elif flag == "module":
                        test_record.is_module = True
        elif key == "includes":
            test_record.includes = value


def _find_attrs(src: Text) -> Tuple[Optional[Text], Optional[Text]]:
    match = _YAML_PATTERN.search(src)
    if not match:
        return None, None

    return match.group(0), match.group(1).strip()


def parse(logger: Logger, src: Text, name: Text) -> Optional[TestRecord]:
    if name.endswith('_FIXTURE.js'):
        return None

    # Find the YAML frontmatter.
    frontmatter, attrs = _find_attrs(src)

    # YAML frontmatter is required for all tests.
    if frontmatter is None:
        logger.error("Missing frontmatter: %s" % name)
        return None

    test_record = TestRecord(test = src)

    if attrs:
        _yaml_attr_parser(logger, test_record, attrs, name)

    return test_record
