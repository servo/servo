from typing import Any, Dict, IO
from ..meta.schema import SchemaValue

import yaml

# PyYaml does not currently handle unique keys.
# https://github.com/yaml/pyyaml/issues/165#issuecomment-430074049
# In that issue, there are workarounds to it.
# https://gist.github.com/pypt/94d747fe5180851196eb?permalink_comment_id=4015118#gistcomment-4015118

class UniqueKeyLoader(yaml.SafeLoader):
    def construct_mapping(self, node: yaml.MappingNode, deep: bool = False) -> Dict[Any, Any]:
        mapping = set()
        for key_node, value_node in node.value:
            key = self.construct_object(key_node, deep=deep)  # type: ignore
            if key in mapping:
                raise ValueError(f"Duplicate {key!r} key found in YAML.")
            mapping.add(key)
        return super().construct_mapping(node, deep)

def load_data_to_dict(f: IO[bytes]) -> Dict[str, Any]:
    try:
        raw_data = yaml.load(f, Loader=UniqueKeyLoader)
        return SchemaValue.from_dict(raw_data)
    except Exception as e:
        raise e
