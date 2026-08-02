from enum import Enum
from dataclasses import dataclass
from fnmatch import fnmatchcase
from typing import Any, Dict, List, Sequence, Union

from ..schema import SchemaValue, validate_dict

"""
YAML filename for meta files
"""
WEB_FEATURES_YML_FILENAME = "WEB_FEATURES.yml"


class SpecialFileEnum(Enum):
    """All files recursively"""
    RECURSIVE = "**"


class FeatureFile(str):
    def match_files(self, base_filenames: Sequence[str]) -> Sequence[str]:
        """
        Given the input base file names, returns the subset of base file names
        that match the given FeatureFile based on matching_mode.
        If the FeatureFile contains any number of "*" characters, fnmatch is
        used check each file name.
        If the FeatureFile does not contain any "*" characters, the base file name
        must match the FeatureFile exactly
        :param base_filenames: The list of filenames to check against the FeatureFile
        :return: List of matching file names that match FeatureFile
        """
        result = []
        # If our file name contains a wildcard, use fnmatch
        if "*" in self:
            for base_filename in base_filenames:
                if fnmatchcase(base_filename, self):
                    result.append(base_filename)
        elif self in base_filenames:
            result.append(self)
        return result


@dataclass
class FeatureEntry:
    file: Union[FeatureFile, SpecialFileEnum]
    """The web-features key"""
    feature_ids: List[str]

    _required_keys = {"ids"}

    def __init__(self, obj: Dict[str, Union[List[str], Dict[str, List[str]]]]):
        """
        Converts the provided dictionary to an instance of FeatureEntry
        :param obj: The object that will be converted to a FeatureEntry.
        :return: An instance of FeatureEntry
        :raises ValueError: If there are unexpected keys or missing required keys.
        """
        if len(obj) == 0:
            raise ValueError(f"Input value {obj} contains zero keys")

        if len(obj) > 1:
            raise ValueError(f"Input value {obj} contains more than one key")
        key = list(obj)[0]
        self.file = SchemaValue.from_union([
            SpecialFileEnum,
            SchemaValue.from_class(FeatureFile)
        ], key)

        value = obj[key]
        if isinstance(value, list):
            self.feature_ids = value
        else:
            validate_dict(value, FeatureEntry._required_keys)
            self.feature_ids = value["ids"]

    def __str__(self) -> str:
        return '{}: {}'.format(self.file, self.feature_ids)

    def does_feature_apply_recursively(self) -> bool:
        if isinstance(self.file, SpecialFileEnum) and self.file == SpecialFileEnum.RECURSIVE:
            return True
        return False


@dataclass
class WebFeaturesFile:
    """List of features"""
    rules: Sequence[FeatureEntry]

    _required_keys = {"rules"}

    def __init__(self, obj: Dict[str, Any]):
        """
        Converts the provided dictionary to an instance of WebFeaturesFile
        :param obj: The object that will be converted to a WebFeaturesFile.
        :return: An instance of WebFeaturesFile
        :raises ValueError: If there are unexpected keys or missing required keys.
        """
        validate_dict(obj, WebFeaturesFile._required_keys)
        self.rules = SchemaValue.from_list(
            lambda raw_feature: FeatureEntry(SchemaValue.from_dict(raw_feature)), obj.get("rules"))
