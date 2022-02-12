def load_pdf_document(session, inline, pdf_data):
    """Load a PDF document in the browser using pdf.js"""
    session.url = inline("""
<!doctype html>
<script src="/_pdf_js/pdf.js"></script>
<canvas></canvas>
<script>
async function getText() {
  pages = [];
  let loadingTask = pdfjsLib.getDocument({data: atob("%s")});
  let pdf = await loadingTask.promise;
  for (let pageNumber=1; pageNumber<=pdf.numPages; pageNumber++) {
    let page = await pdf.getPage(pageNumber);
    textContent = await page.getTextContent()
    text = textContent.items.map(x => x.str).join("");
    pages.push(text);
  }
  return pages
}
</script>
""" % pdf_data)
