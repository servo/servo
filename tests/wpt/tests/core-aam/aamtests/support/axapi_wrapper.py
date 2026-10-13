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

class AxapiWrapper(ApiWrapper[AXUIElement, Any]):

    @property
    def api_name(self) -> str:
        return "AXAPI"

    @property
    def AXUIElementCopyAttributeValue(self):
        return AXUIElementCopyAttributeValue

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

    def get_dom_identifiers(self, elements: List[AXUIElement]) -> List[str]:
        """Given AXUIElements, return their AXDOMIdentifier values.

        :param elements: AXUIElements to look up.
        :return: A list of DOM ids in the same order as the input.
        """
        ids: List[str] = []
        for element in elements:
            err, value = AXUIElementCopyAttributeValue(
                element, "AXDOMIdentifier", None
            )
            ids.append(value if not err else "[unknown id]")
        return ids
