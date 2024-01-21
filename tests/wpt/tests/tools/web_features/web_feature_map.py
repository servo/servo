import itertools

from collections import OrderedDict
from os.path import basename
from typing import Dict, List, Optional, Sequence, Set

from ..manifest.item import ManifestItem, URLManifestItem
from ..manifest.sourcefile import SourceFile
from ..metadata.webfeatures.schema import FeatureEntry, FeatureFile, WebFeaturesFile


class WebFeaturesMap:
    """
    Stores a mapping of web-features to their associated test paths.
    """

    def __init__(self) -> None:
        """
        Initializes the WebFeaturesMap with an OrderedDict to maintain feature order.
        """
        self._feature_tests_map_: OrderedDict[str, Set[str]] = OrderedDict()


    def add(self, feature: str, manifest_items: List[ManifestItem]) -> None:
        """
        Adds a web feature and its associated test paths to the map.

        Args:
            feature: The web-features identifier.
            manifest_items: The ManifestItem objects representing the test paths.
        """
        tests = self._feature_tests_map_.get(feature, set())
        self._feature_tests_map_[feature] = tests.union([
            manifest_item.url for manifest_item in manifest_items if isinstance(manifest_item, URLManifestItem)])


    def to_dict(self) -> Dict[str, List[str]]:
        """
        Returns:
            The plain dictionary representation of the map.
        """
        rv: Dict[str, List[str]] = {}
        for feature, manifest_items in self._feature_tests_map_.items():
            # Sort the list to keep output stable
            rv[feature] = sorted(manifest_items)
        return rv


class WebFeatureToTestsDirMapper:
    """
    Maps web-features to tests within a specified directory.
    """

    def __init__(
            self,
            all_test_files_in_dir: List[SourceFile],
            web_feature_file: Optional[WebFeaturesFile]):
        """
        Initializes the mapper with test paths and web feature information.
        """

        self.all_test_files_in_dir = all_test_files_in_dir
        self.test_path_to_manifest_items_map = dict([(basename(f.path), f.manifest_items()[1]) for f in self.all_test_files_in_dir])
        # Used to check if the current directory has a WEB_FEATURE_FILENAME
        self.web_feature_file = web_feature_file
        # Gets the manifest items for each test path and returns them into a single list.
        self. get_all_manifest_items_for_dir = list(itertools.chain.from_iterable([
            items for _, items in self.test_path_to_manifest_items_map.items()]))


    def _process_inherited_features(
            self,
            inherited_features: List[str],
            result: WebFeaturesMap) -> None:
        # No WEB_FEATURE.yml in this directory. Simply add the current features to the inherited features
        for inherited_feature in inherited_features:
            result.add(inherited_feature, self.get_all_manifest_items_for_dir)

    def _process_recursive_feature(
            self,
            inherited_features: List[str],
            feature: FeatureEntry,
            result: WebFeaturesMap) -> None:
        inherited_features.append(feature.name)
        result.add(feature.name, self.get_all_manifest_items_for_dir)

    def _process_non_recursive_feature(
            self,
            feature_name: str,
            files: Sequence[FeatureFile],
            result: WebFeaturesMap) -> None:
        # If the feature does not apply recursively, look at the individual
        # files and match them against all_test_files_in_dir.
        test_file_paths: List[ManifestItem] = []
        base_test_file_names = [basename(f.path) for f in self.all_test_files_in_dir]
        for test_file in files:
            matched_base_file_names = test_file.match_files(base_test_file_names)
            test_file_paths.extend(itertools.chain.from_iterable([
                self.test_path_to_manifest_items_map[f] for f in matched_base_file_names]))

        result.add(feature_name, test_file_paths)

    def run(self, result: WebFeaturesMap, inherited_features: List[str]) -> None:
        if self.web_feature_file:
            # Do not copy the inherited features because the presence of a
            # WEB_FEATURES.yml file indicates new instructions.
            inherited_features.clear()

            # Iterate over all the features in this new file
            for feature in self.web_feature_file.features:
                # Handle the "**" case
                if feature.does_feature_apply_recursively():
                    self._process_recursive_feature(inherited_features, feature, result)

                # Handle the non recursive case.
                elif isinstance(feature.files, List) and feature.files:
                    self._process_non_recursive_feature(feature.name, feature.files, result)
        else:
            self._process_inherited_features(inherited_features, result)
