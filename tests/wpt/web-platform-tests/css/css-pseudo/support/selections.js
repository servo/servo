/**
 * Replaces the current selection (if any) with a new range, after
 * it’s configured by the given function.
 */
function selectRangeWith(fun) {
    const selection = getSelection();

    // Deselect any ranges that happen to be selected, to prevent the
    // Selection#addRange call from ignoring our new range (see
    // <https://www.chromestatus.com/feature/6680566019653632> for
    // more details).
    selection.removeAllRanges();

    // Create and configure a range.
    const range = document.createRange();
    fun(range);

    // Select our new range.
    selection.addRange(range);
}

/**
 * Replaces the current selection (if any) with a new range, spanning
 * the contents of the given node.
 */
function selectNodeContents(node) {
    const previousActive = document.activeElement;

    selectRangeWith(range => range.selectNodeContents(node));

    // If the selection update causes the node or an ancestor to be
    // focused (Chromium 80+), unfocus it, to avoid any focus-related
    // styling such as outlines.
    if (document.activeElement != previousActive) {
        document.activeElement.blur();
    }
}

/**
 * Tries to convince a UA with lazy spellcheck to check and mark the
 * contents of the given nodes (form fields or @contenteditables).
 *
 * Both focus and selection can be used for this purpose, but only
 * focus works for @contenteditables.
 */
function trySpellcheck(...nodes) {
    // This is inherently a flaky test risk, but Chromium (as of 87)
    // seems to cancel spellcheck on a node if it wasn’t the last one
    // focused for “long enough” (though immediate unfocus is ok).
    // setInterval(0) is usually not long enough.
    const interval = setInterval(() => {
        if (nodes.length > 0) {
            const node = nodes.shift();
            node.focus();
            node.blur();
        } else {
            clearInterval(interval);
        }
    }, 250);
}
