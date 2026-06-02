from dataclasses import dataclass
from typing import Any, ClassVar, Dict, List, Optional, Set

from ..schema import SchemaValue, validate_dict

"""
YAML filename for meta files
"""
META_YML_FILENAME = "META.yml"

@dataclass
class MetaFile():
    """documented structure of META files.
    Reference: https://github.com/web-platform-tests/wpt/pull/18434
    """

    """a link to the specification covered by the tests in the directory"""
    spec: Optional[str] = None
    """a list of GitHub account username belonging to people who are notified when pull requests
    modify files in the directory
    """
    suggested_reviewers: Optional[List[str]] = None

    _optional_keys: ClassVar[Set[str]] = {"spec", "suggested_reviewers"}

    def __init__(self, obj: Dict[str, Any]):
        validate_dict(obj, optional_keys=MetaFile._optional_keys)
        self.spec = SchemaValue.from_union([SchemaValue.from_str, SchemaValue.from_none], obj.get("spec"))
        self.suggested_reviewers = SchemaValue.from_union(
            [lambda x: SchemaValue.from_list(SchemaValue.from_str, x), SchemaValue.from_none],
            obj.get("suggested_reviewers"))
