setup({explicit_done:true});

function t(desc, func, expect) {
  async_test(function() {
    var img = document.querySelector('[data-desc="' + desc + '"]');
    img.onload = img.onerror = this.unreached_func('update the image data was run');
    if (expect == 'timeout') {
      setTimeout(this.step_func_done(), 1000);
    } else {
      img['on' + expect] = this.step_func_done();
      setTimeout(this.unreached_func('update the image data didn\'t run'), 1000);
    }
    func.call(this, img);
  }, desc);
}
