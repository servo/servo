from enum import Enum
from dataclasses import dataclass
from fnmatch import fnmatchcase
from functools import cached_property
from typing import Any, Dict, Sequence, Union

from ..schema import SchemaValue, validate_dict

"""
YAML filename for meta files
"""
WEB_FEATURES_YML_FILENAME = "WEB_FEATURES.yml"

# File prefix to indicate that this FeatureFile should run in EXCLUDE mode.
EXCLUSION_PREFIX = "!"


class SpecialFileEnum(Enum):
    """All files recursively"""
    RECURSIVE = "**"


class FileMatchingMode(Enum):
    """Defines how a FeatureFile pattern is used for matching."""
    INCLUDE = 1  # Include files that match the pattern
    EXCLUDE = 2  # Exclude files that match the pattern

class FeatureFile(str):
    @cached_property
    def matching_mode(self) -> FileMatchingMode:
        """Determines if the pattern should include or exclude matches."""
        return FileMatchingMode.EXCLUDE if self.startswith(EXCLUSION_PREFIX) else FileMatchingMode.INCLUDE

    @cached_property
    def processed_filename(self) -> str:
        """Removes the exclusion prefix "!" from the pattern."""
        # TODO. After moving to Python3.9, use: return self.removeprefix(EXCLUSION_PREFIX)
        return self[len(EXCLUSION_PREFIX):] if self.startswith(EXCLUSION_PREFIX) else self

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
                if fnmatchcase(base_filename, self.processed_filename):
                    result.append(base_filename)
        elif self.processed_filename in base_filenames:
            result.append(self.processed_filename)
        return result


@dataclass
class FeatureEntry:
    files: Union[Sequence[FeatureFile], SpecialFileEnum]
    """The web-features key"""
    name: str

    _required_keys = {"files", "name"}

    def __init__(self, obj: Dict[str, Any]):
        """
        Converts the provided dictionary to an instance of FeatureEntry
        :param obj: The object that will be converted to a FeatureEntry.
        :return: An instance of FeatureEntry
        :raises ValueError: If there are unexpected keys or missing required keys.
        """
        validate_dict(obj, FeatureEntry._required_keys)
        self.files = SchemaValue.from_union([
            lambda x: SchemaValue.from_list(SchemaValue.from_class(FeatureFile), x),
            SpecialFileEnum], obj.get("files"))
        self.name = SchemaValue.from_str(obj.get("name"))


    def does_feature_apply_recursively(self) -> bool:
        if isinstance(self.files, SpecialFileEnum) and self.files == SpecialFileEnum.RECURSIVE:
            return True
        return False


@dataclass
class WebFeaturesFile:
    """List of features"""
    features: Sequence[FeatureEntry]

    _required_keys = {"features"}

    def __init__(self, obj: Dict[str, Any]):
        """
        Converts the provided dictionary to an instance of WebFeaturesFile
        :param obj: The object that will be converted to a WebFeaturesFile.
        :return: An instance of WebFeaturesFile
        :raises ValueError: If there are unexpected keys or missing required keys.
        """
        validate_dict(obj, WebFeaturesFile._required_keys)
        self.features = SchemaValue.from_list(
            lambda raw_feature: FeatureEntry(SchemaValue.from_dict(raw_feature)), obj.get("features"))
