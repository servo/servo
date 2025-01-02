const simpleTestString = `ABCDEFGHIJKLO
AAAAAAAA
AAAA
AA
A`;

function generateSimpleTest({ textGroupAlign, textAlign, direction, writingMode }) {
    if (!direction)
        direction = "ltr";
    if (!writingMode)
        writingMode = "horizontal-tb";

    const container = document.createElement("div");
    container.textContent = simpleTestString;
    container.style = `text-group-align: ${textGroupAlign}; text-align: ${textAlign}; direction: ${direction}; writing-mode: ${writingMode}`;
    container.classList.add("text-container", "pre");
    document.body.append(container);
}

function generateSimpleReference({ textGroupAlign, textAlign, direction, writingMode }) {
    if (!direction)
        direction = "ltr";
    if (!writingMode)
        writingMode = "horizontal-tb";

    const container = document.createElement("div");
    container.classList.add("text-container", "pre");
    container.style = `text-align: ${textAlign}; direction: ${direction}; writing-mode: ${writingMode}`;

    const group = document.createElement("div");
    group.textContent = simpleTestString;
    group.classList.add("group", `group-align-${textGroupAlign}`);
    container.append(group);

    document.body.append(container);
}
