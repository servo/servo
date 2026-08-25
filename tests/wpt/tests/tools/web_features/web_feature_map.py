import itertools

from collections import OrderedDict
from os.path import basename
from typing import Dict, List, Optional, Set, Union

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
        self._feature_tests_map_: OrderedDict[Union[str, None], Set[str]] = OrderedDict()
        self._classified_urls: Set[str] = set()


    def _should_classify(self, manifest_item: URLManifestItem) -> bool:
        return manifest_item.url not in self._classified_urls

    def add(self, feature_ids: List[str], manifest_items: List[ManifestItem]) -> None:
        """
        Adds a web feature and its associated test paths to the map.

        Args:
            feature_ids: The web-features identifier(s).
            manifest_items: The ManifestItem objects representing the test paths.
        """
        urls = []
        for manifest_item in manifest_items:
            if isinstance(manifest_item, URLManifestItem) and self._should_classify(manifest_item):
                urls.append(manifest_item.url)

        self._classified_urls.update(urls)
        for feature_id in feature_ids:
            tests = self._feature_tests_map_.get(feature_id, set())
            self._feature_tests_map_[feature_id] = tests.union(urls)


    def to_dict(self) -> Dict[str, List[str]]:
        """
        Returns:
            The plain dictionary representation of the map.
        """
        rv: Dict[str, List[str]] = {}
        for feature, manifest_items in self._feature_tests_map_.items():
            if feature is None:
                continue
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
        result.add(inherited_features, self.get_all_manifest_items_for_dir)

    def _process_recursive_feature(
            self,
            inherited_features: List[str],
            feature: FeatureEntry,
            result: WebFeaturesMap) -> None:
        inherited_features.extend(feature.feature_ids)
        result.add(feature.feature_ids, self.get_all_manifest_items_for_dir)

    def _process_non_recursive_feature(
            self,
            feature_ids: List[str],
            test_file: FeatureFile,
            result: WebFeaturesMap) -> None:
        # If the feature does not apply recursively, look at the individual
        # files and match them against all_test_files_in_dir.
        final_test_file_paths: List[ManifestItem] = []
        test_file_paths: Set[str] = set()
        base_test_file_names = [basename(f.path) for f in self.all_test_files_in_dir]

        test_file_paths.update(
            test_file.match_files(base_test_file_names)
        )

        final_test_file_paths.extend(itertools.chain.from_iterable([
            self.test_path_to_manifest_items_map[f] for f in test_file_paths]))

        result.add(feature_ids, final_test_file_paths)

    def run(self, result: WebFeaturesMap, inherited_features: List[str]) -> None:
        if self.web_feature_file:
            # Do not copy the inherited features because the presence of a
            # WEB_FEATURES.yml file indicates new instructions.
            inherited_features.clear()

            # Iterate over all the features in this new file
            for rule in self.web_feature_file.rules:
                # Handle the "**" case
                if rule.does_feature_apply_recursively():
                    self._process_recursive_feature(inherited_features, rule, result)

                # Handle the non recursive case.
                elif isinstance(rule.file, FeatureFile):
                    self._process_non_recursive_feature(rule.feature_ids, rule.file, result)
        else:
            self._process_inherited_features(inherited_features, result)
