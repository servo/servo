from __future__ import annotations
from typing import Any, Optional, List, Dict

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib

from .api_wrapper import ApiWrapper

# The roles browsers use for the root of a web document.
DOCUMENT_ROLES = [Atspi.Role.DOCUMENT_WEB, Atspi.Role.DOCUMENT_FRAME]

# The document attributes browsers use to expose the url of a web document.
# Firefox uses "DocURL", Chromium family browsers use "URI".
DOCUMENT_URL_ATTRIBUTES = ["DocURL", "URI"]


class AtspiWrapper(ApiWrapper[Atspi.Accessible]):

    @property
    def api_name(self) -> str:
        return "ATSPI"

    def __getattr__(self, name: str) -> Any:
        return getattr(Atspi, name)

    def find_node(self, dom_id: str, url: str) -> Atspi.Accessible:
        """
        :param dom_id: The dom id of the node to test.
        :param url: The url of the test.
        """
        if self.test_url != url or not self.document:
            self.test_url = url
            self.document = self._poll_for(
                self._find_fully_loaded_document, f"Timeout looking for url: {self.test_url}"
            )

        test_node = self._find_node_by_id(self.document, dom_id);
        if not test_node:
            raise Exception(f"Did not find node with id '{dom_id}' in accessibility API ATSPI.")

        return test_node

    def get_relations_dictionary_helper(
        self, node: Atspi.Accessible
    ) -> Dict[str, List[str]]:
        """
        :returns: A dictionary with relations as keys and the values, DOM ids.
        """
        relations_dict: Dict[str, List[str]] = {}
        relations = Atspi.Accessible.get_relation_set(node)
        for relation in relations:
            name = relation.get_relation_type().value_name.removeprefix("ATSPI_")
            relations_dict[name] = []
            num_targets = relation.get_n_targets()

            for i in range(num_targets):
                target = relation.get_target(i)
                attributes = Atspi.Accessible.get_attributes(target)
                relations_dict[name].append(attributes.get("id", "[unknown id]"))

        return relations_dict

    def get_state_list_helper(self, node: Atspi.Accessible) -> List[str]:
        """
        :returns: A list of states for this Atspi.Accessible.
        """
        state_list = Atspi.Accessible.get_state_set(node).get_states()
        return [state.value_name.removeprefix("ATSPI_") for state in state_list]

    def _find_browser(self) -> Optional[Atspi.Accessible]:
        if self.pid and self.pid != 0:
            return self._find_browser_by_pid()
        else:
            return self._find_browser_by_name()

    def _find_browser_by_pid(self) -> Optional[Atspi.Accessible]:
        """Find the Atspi.Accessible representing the browser.

        :param pid: The PID of the browser.
        :return: Atspi.Accessible or None.
        """
        desktop = Atspi.get_desktop(0)
        child_count = Atspi.Accessible.get_child_count(desktop)
        for i in range(child_count):
            app = Atspi.Accessible.get_child_at_index(desktop, i)
            if self.pid == Atspi.Accessible.get_process_id(app):
                return app
        return None

    def _find_browser_by_name(self) -> Optional[Atspi.Accessible]:
        """Find the Atspi.Accessible representing the browser.

        :param name: The name of the browser.
        :return: Atspi.Accessible or None.
        """
        desktop = Atspi.get_desktop(0)
        child_count = Atspi.Accessible.get_child_count(desktop)
        for i in range(child_count):
            app = Atspi.Accessible.get_child_at_index(desktop, i)
            full_app_name = Atspi.Accessible.get_name(app)
            if self.product_name in full_app_name.lower():
                return app
        return None

    def _find_fully_loaded_document(self) -> Optional[Atspi.Accessible]:
        """Find the document with the test url. Only returns it when it is ready.

        :return: Atspi.Accessible representing test document or None.
        """
        for document in self._find_documents():
            if self._is_ready(document, self.test_url):
                return document

        return None

    def _find_documents(self) -> List[Atspi.Accessible]:
        """Find the web documents the browser exposes.

        :return: A list of Atspi.Accessible, which may be empty.
        """
        documents = self._query_documents(self.root)
        if documents is None:
            documents = self._walk_documents(self.root)

        return documents

    def _query_documents(
        self, root: Atspi.Accessible
    ) -> Optional[List[Atspi.Accessible]]:
        """Find the web documents in a subtree with the Collection interface.

        The Collection interface lets the browser do the searching, which saves
        walking the whole tree over the bus.

        :param root: The root node to search from.
        :return: A list of Atspi.Accessible, or None if the browser does not
                 answer Collection queries.
        """
        collection = Atspi.Accessible.get_collection_iface(root)
        if not collection:
            return None

        rule = Atspi.MatchRule.new(
            Atspi.StateSet.new([]),
            Atspi.CollectionMatchType.ALL,
            {},
            Atspi.CollectionMatchType.ALL,
            DOCUMENT_ROLES,
            Atspi.CollectionMatchType.ANY,
            [],
            Atspi.CollectionMatchType.ALL,
            False,
        )

        try:
            return Atspi.Collection.get_matches(
                collection, rule, Atspi.CollectionSortOrder.CANONICAL, 0, True
            )
        except GLib.Error:
            return None

    def _walk_documents(self, root: Atspi.Accessible) -> List[Atspi.Accessible]:
        """Find the web documents in a subtree by walking the tree.

        :param root: The root node to search from.
        :return: A list of Atspi.Accessible, which may be empty.
        """
        documents = []
        stack = [root]
        while stack:
            node = stack.pop()
            if Atspi.Accessible.get_role(node) in DOCUMENT_ROLES:
                documents.append(node)
                continue

            for i in range(Atspi.Accessible.get_child_count(node)):
                child = Atspi.Accessible.get_child_at_index(node, i)
                stack.append(child)

        return documents

    def _is_ready(self, document: Atspi.Accessible, url: str) -> bool:
        """Test whether a document is the test document, fully loaded.

        :param document: Atspi.Accessible representing a web document.
        :param url: The url of the test.
        :return: Boolean.
        """
        # Firefox uses the "BUSY" state to indicate the page is not ready.
        if self.product_name == "firefox":
            state_set = Atspi.Accessible.get_state_set(document)
            if Atspi.StateSet.contains(state_set, Atspi.StateType.BUSY):
                return False

        # The url tells the test document apart from the other documents the
        # browser exposes, such as one a previous navigation left behind.
        # Chromium family browsers do not use "BUSY", but only expose the url
        # once the document can be queried. If it is not here, we need to query
        # for a new accessible object.
        return self._document_url(document) == url

    def _document_url(self, document: Atspi.Accessible) -> Optional[str]:
        """Get the url a web document was loaded from.

        :param document: Atspi.Accessible representing a web document.
        :return: The url, or None if the document does not expose one.
        """
        document_iface = Atspi.Accessible.get_document_iface(document)
        if not document_iface:
            return None

        attributes = Atspi.Document.get_document_attributes(document_iface)
        for name in DOCUMENT_URL_ATTRIBUTES:
            if name in attributes:
                return attributes[name]

        return None

    def _find_node_by_id(
        self, root: Atspi.Accessible, dom_id: str
    ) -> Optional[Atspi.Accessible]:
        """Find the Atspi.Accessible with a specified dom_id.

        :param root: The root node to search from.
        :param dom_id: The dom ID.
        :return: Atspi.Accessible or None if not found.
        """
        stack = [root]
        while stack:
            node = stack.pop()
            attributes = Atspi.Accessible.get_attributes(node)
            if "id" in attributes and attributes["id"] == dom_id:
                return node

            for i in range(Atspi.Accessible.get_child_count(node)):
                child = Atspi.Accessible.get_child_at_index(node, i)
                stack.append(child)

        return None
