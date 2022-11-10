import pytest

@pytest.fixture
def checkbox_dom(inline):
    return """
        <style>
            custom-checkbox-element {
                display:block; width:20px; height:20px;
            }
        </style>
        <custom-checkbox-element></custom-checkbox-element>
        <script>
            customElements.define('custom-checkbox-element',
                class extends HTMLElement {
                    constructor() {
                            super();
                            this.attachShadow({mode: 'open'}).innerHTML = `
                                <div><input type="checkbox"/></div>
                            `;
                        }
                });
        </script>"""
