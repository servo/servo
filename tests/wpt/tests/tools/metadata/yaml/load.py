from typing import Any, Dict, IO
from ..meta.schema import SchemaValue

import yaml

def load_data_to_dict(f: IO[bytes]) -> Dict[str, Any]:
    try:
        raw_data = yaml.safe_load(f)
        return SchemaValue.from_dict(raw_data)
    except Exception as e:
        raise e
