from __future__ import annotations
from typing import Any, Optional, List, Dict

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi

from .api_wrapper import ApiWrapper


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
                self._find_fully_loaded_tab, f"Timeout looking for url: {self.test_url}"
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

    def _find_fully_loaded_tab(self) -> Optional[Atspi.Accessible]:
        """Find the tab with the test url. Only returns the tab when the tab is ready.

        :param url: The url of the test.
        :return: Atspi.Accessible representing test document or None.
        """
        stack = [self.root]
        while stack:
            node = stack.pop()
            if Atspi.Accessible.get_role_name(node) == "frame":
                relationset = Atspi.Accessible.get_relation_set(node)
                for relation in relationset:
                    if relation.get_relation_type() == Atspi.RelationType.EMBEDS:
                        tab = relation.get_target(0)
                        if self._is_ready(tab, self.test_url):
                            return tab
                        else:
                            return None
                continue

            for i in range(Atspi.Accessible.get_child_count(node)):
                child = Atspi.Accessible.get_child_at_index(node, i)
                stack.append(child)

        return None

    def _is_ready(self, tab: Atspi.Accessible, url: str) -> bool:
        """Test whether tab is fully loaded.

        :param tab: Atspi.Accessible representing test document.
        :param url: The url of the test.
        :return: Boolean.
        """
        # Firefox uses the "BUSY" state to indicate the page is not ready.
        if self.product_name == "firefox":
            state_set = Atspi.Accessible.get_state_set(tab)
            return not Atspi.StateSet.contains(state_set, Atspi.StateType.BUSY)

        # Chromium family browsers do not use "BUSY", but you can
        # tell if the document can be queried by URL attribute. If the 'URI'
        # attribute is not here, we need to query for a new accessible object.
        document = Atspi.Accessible.get_document_iface(tab)
        document_attributes = Atspi.Document.get_document_attributes(document)

        return "URI" in document_attributes and document_attributes["URI"] == url

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
