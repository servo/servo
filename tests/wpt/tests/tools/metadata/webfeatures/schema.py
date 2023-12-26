from enum import Enum
from dataclasses import dataclass
from fnmatch import fnmatchcase
from typing import Any, Dict, Sequence, Union

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
        that match the given FeatureFile.
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
        elif self.__str__() in base_filenames:
            result.append(self)
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
