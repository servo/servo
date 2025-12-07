import pytest
from typing import Any, Mapping
from webdriver.bidi.modules.script import ContextTarget
@pytest.fixture
def add_speculation_rules_and_link(bidi_session):
    """Helper for adding both speculation rules and a prefetch link to a page."""
    async def add_rules_and_link(context: Mapping[str, Any], rules: str, href: str, text: str = "Test Link", link_id: str = "prefetch-page"):
        """Add speculation rules and a corresponding link to the page."""
        # Add speculation rules first
        await bidi_session.script.evaluate(
            expression=f"""
                const script = document.createElement('script');
                script.type = 'speculationrules';
                script.textContent = `{rules}`;
                document.head.appendChild(script);
            """,
            target=ContextTarget(context["context"]),
            await_promise=False
        )
        # Then add the link
        await bidi_session.script.evaluate(
            expression=f"""
                const link = document.createElement('a');
                link.href = '{href}';
                link.textContent = '{text}';
                link.id = '{link_id}';
                document.body.appendChild(link);
            """,
            target={"context": context["context"]},
            await_promise=False
        )
    return add_rules_and_link