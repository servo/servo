from __future__ import annotations

from typing import Any, Optional

from ApplicationServices import (
    AXUIElementCopyAttributeNames,
    AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication,
)

from Cocoa import (
    NSApplicationActivationPolicyRegular,
    NSPredicate,
    NSWorkspace,
)

from .api_wrapper import ApiWrapper

AXUIElement = Any

class AxapiWrapper(ApiWrapper[AXUIElement]):

    @property
    def api_name(self) -> str:
        return "AXAPI"

    @property
    def AXUIElementCopyAttributeValue(self):
        return AXUIElementCopyAttributeValue

    def find_node(self, dom_id: str, url: str) -> AXUIElement:
        """
        :param dom_id: The dom id of the node to test.
        :param url: The url of the test.
        """
        if self.test_url != url or not self.document:
            self.test_url = url
            self.document = self._poll_for(
                self._find_tab,
                f"Timeout looking for url: {self.test_url}",
            )

        test_node = self._poll_for(
            lambda: self._find_node_by_id(self.document, dom_id),
            f"Timeout looking for node with id {dom_id} in accessibility API AXAPI.",
        )

        return test_node

    def _find_browser(self) -> Optional[AXUIElement]:
        """Find the AXUIElement representing the browser.

        :return: AXUIElement or None.
        """
        if self.pid and self.pid != 0:
            return AXUIElementCreateApplication(self.pid)

        ws = NSWorkspace.sharedWorkspace()
        regular_predicate = NSPredicate.predicateWithFormat_(
            f"activationPolicy == {NSApplicationActivationPolicyRegular}"
        )
        running_apps = ws.runningApplications().filteredArrayUsingPredicate_(
            regular_predicate
        )
        name_predicate = NSPredicate.predicateWithFormat_(
            f"localizedName contains[c] '{self.product_name}'"
        )
        filtered_apps = running_apps.filteredArrayUsingPredicate_(name_predicate)
        if filtered_apps.count() == 0:
            return None
        app = filtered_apps[0]
        pid = app.processIdentifier()
        if pid == -1:
            return None
        return AXUIElementCreateApplication(pid)

    def _find_tab(self) -> Optional[AXUIElement]:
        """Find the active tab of the browser.

        :return: AXUIElement representing test document or None.
        """
        stack = [self.root]
        while stack:
            node = stack.pop()

            err, role = AXUIElementCopyAttributeValue(node, "AXRole", None)
            if err:
                continue
            if role == "AXWebArea":
                # TODO: AtspiWrapper will check that the found tab is the correct
                # tab by checking the URL. Perform this check here.
                return node

            err, children = AXUIElementCopyAttributeValue(node, "AXChildren", None)
            if err:
                continue
            stack.extend(children)

        return None

    def _find_node_by_id(self, root: Any, dom_id: str) -> Optional[AXUIElement]:
        """Find the AXUIElement with a specified dom_id.

        :param root: The root node to search from.
        :param dom_id: The dom ID.
        :return: AXUIElement or None if not found.
        """
        stack = [root]
        while stack:
            node = stack.pop()

            err, attributes = AXUIElementCopyAttributeNames(node, None)
            if err:
                continue
            if "AXDOMIdentifier" in attributes:
                err, value = AXUIElementCopyAttributeValue(
                    node, "AXDOMIdentifier", None
                )
                if not err and value == dom_id:
                    return node

            err, children = AXUIElementCopyAttributeValue(node, "AXChildren", None)
            if err:
                continue
            stack.extend(children)

        return None
