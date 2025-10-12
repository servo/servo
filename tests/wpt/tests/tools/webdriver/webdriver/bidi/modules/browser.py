from typing import Any, List, Mapping

from ._module import BidiModule, command
from ..undefined import UNDEFINED, Maybe, Nullable


class Browser(BidiModule):
    @command
    def close(self) -> Mapping[str, Any]:
        return {}

    @command
    def get_client_windows(self) -> Mapping[str, Any]:
        return {}

    @get_client_windows.result
    def _get_client_windows(self, result: Mapping[str, Any]) -> Any:
        assert result['clientWindows'] is not None
        assert isinstance(result["clientWindows"], list)
        for client_window_info in result["clientWindows"]:
            assert isinstance(client_window_info["active"], bool)
            assert isinstance(client_window_info["clientWindow"], str)
            assert isinstance(client_window_info["state"], str)
            assert isinstance(client_window_info["height"], int)
            assert isinstance(client_window_info["width"], int)
            assert isinstance(client_window_info["x"], int)
            assert isinstance(client_window_info["y"], int)
        return result["clientWindows"]

    @command
    def create_user_context(
            self, accept_insecure_certs: Maybe[bool] = UNDEFINED,
            proxy: Maybe[Mapping[str, Any]] = UNDEFINED,
            unhandled_prompt_behavior: Maybe[Mapping[str, str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "acceptInsecureCerts": accept_insecure_certs,
            "proxy": proxy,
            "unhandledPromptBehavior": unhandled_prompt_behavior
        }

    @create_user_context.result
    def _create_user_context(self, result: Mapping[str, Any]) -> Any:
        assert result["userContext"] is not None
        assert isinstance(result["userContext"], str)

        return result["userContext"]

    @command
    def get_user_contexts(self) -> Mapping[str, Any]:
        return {}

    @get_user_contexts.result
    def _get_user_contexts(self, result: Mapping[str, Any]) -> Any:
        assert result["userContexts"] is not None
        assert isinstance(result["userContexts"], list)
        for user_context_info in result["userContexts"]:
            assert isinstance(user_context_info["userContext"], str)

        return result["userContexts"]

    @command
    def remove_user_context(
            self, user_context: str
    ) -> Mapping[str, Any]:
        return {
            "userContext": user_context
        }

    @command
    def set_download_behavior(
            self, download_behavior: Nullable[Mapping[str, Any]],
            user_contexts: Maybe[List[str]] = UNDEFINED

    ) -> Mapping[str, Any]:
        return {
            "downloadBehavior": download_behavior,
            "userContexts": user_contexts
        }
