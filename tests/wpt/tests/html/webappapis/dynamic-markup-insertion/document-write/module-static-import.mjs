document.write("document.write body contents\n");
document.close();

window.parent.document.dispatchEvent(new CustomEvent("documentWriteDone"));
