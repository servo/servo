var OperatorDictionaryTests = {
    "lspace/rspace": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        assert_true(MathMLFeatureDetection.has_operator_spacing());
        document.body.insertAdjacentHTML("beforeend", `<div>\
lspace/rspace for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <mrow>\
    <mn>&nbsp;</mn>\
    <mo form="${parsedKey.form}">${parsedKey.characters}</mo>\
    <mn>&nbsp;</mn>\
  </mrow>\
</math>\
 VS \
<math>\
  <mrow>\
    <mn>&nbsp;</mn>\
    <mo form="${parsedKey.form}" lspace="${defaultPropertyValue(entry, 'lspace')}" rspace="${defaultPropertyValue(entry, 'rspace')}">${parsedKey.characters}</mo>\
    <mn>&nbsp;</mn>\
  </mrow>\
</math>\
</div>`);
        var div = document.body.lastElementChild;
        var mrows = div.getElementsByTagName("mrow");
        function spaceBetween(element, i, j) {
            return element.children[j].getBoundingClientRect().left -
                element.children[i].getBoundingClientRect().right;
        }
        var lspace = spaceBetween(mrows[0], 0, 1);
        var rspace = spaceBetween(mrows[0], 1, 2);
        var lspaceRef = spaceBetween(mrows[1], 0, 1);
        var rspaceRef = spaceBetween(mrows[1], 1, 2);
        assert_approx_equals(lspace, lspaceRef, epsilon, `lspace (${key})`);
        assert_approx_equals(rspace, rspaceRef, epsilon, `rspace (${key})`);
        div.style.display = "none";
    },

    "movablelimits": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        assert_true(MathMLFeatureDetection.has_movablelimits());
        var defaultValue = defaultPropertyValue(entry, "movablelimits");
        document.body.insertAdjacentHTML("beforeend", `<div>\
movablelimits for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <munder>\
    <mo stretchy="false" form="${parsedKey.form}">${parsedKey.characters}</mo>\
    <mn>&nbsp;</mn>\
  </munder>\
</math>\
 VS \
<math>\
  <munder>\
    <mo stretchy="false" form="${parsedKey.form}" movablelimits="${defaultValue}">${parsedKey.characters}</mo>\
    <mn>&nbsp;</mn>\
  </munder>\
</math>\
</div>`);
        var div = document.body.lastElementChild;
        var munders = div.getElementsByTagName("munder");
        munder = munders[0].getBoundingClientRect()
        munderRef = munders[1].getBoundingClientRect()
        assert_approx_equals(munder.height, munderRef.height, epsilon, `Movablelimits property for ${key} should be '${defaultValue}'`);
        div.style.display = "none";
    },

    "largeop": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        // FIXME: Should really detect largeop support...
        assert_true(MathMLFeatureDetection.has_mspace());
        var defaultValue = defaultPropertyValue(entry, "largeop");
        document.body.insertAdjacentHTML("beforeend", `<div>\
largeop for "${parsedKey.characters}" (${parsedKey.form}): \
<math displaystyle="true">\
  <mo form="${parsedKey.form}">${parsedKey.characters}</mo>\
</math>\
 VS \
<math displaystyle="true">\
  <mo form="${parsedKey.form}" largeop="${defaultValue}">${parsedKey.characters}</mo>\
</math>\
</div>`);
        var div = document.body.lastElementChild;
        var mos = div.getElementsByTagName("mo");
        mo = mos[0].getBoundingClientRect()
        moRef = mos[1].getBoundingClientRect()
        assert_approx_equals(mo.height, moRef.height, epsilon, `Largeop property for ${key} should be '${defaultValue}'`);
        div.style.display = "none";
    },

    "stretchy": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        if (entry.horizontal) {
            // FIXME: Should really detect stretchy support...
            assert_true(MathMLFeatureDetection.has_munder());
            var defaultValue = defaultPropertyValue(entry, "stretchy");
            document.body.insertAdjacentHTML("beforeend", `<div>\
stretchy for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <munder>\
    <mn>&nbsp;&nbsp;</mn>\
    <mo form="${parsedKey.form}">${parsedKey.characters}</mo>\
  </munder>\
</math>\
 VS \
<math>\
  <munder>\
    <mn>&nbsp;&nbsp;</mn>\
    <mo form="${parsedKey.form}" stretchy="${defaultValue}">${parsedKey.characters}</mo>\
  </munder>\
</math>\
</div>`);
            var div = document.body.lastElementChild;
            var mos = div.getElementsByTagName("mo");
            mo = mos[0].getBoundingClientRect()
            moRef = mos[1].getBoundingClientRect()
            assert_approx_equals(mo.width, moRef.width, epsilon, `Stretchy property for ${key} should be '${defaultValue}'`);
            div.style.display = "none";
        } else {
            // FIXME: Should really detect stretchy support...
            assert_true(MathMLFeatureDetection.has_mspace());
            var defaultValue = defaultPropertyValue(entry, "stretchy");
            document.body.insertAdjacentHTML("beforeend", `<div>\
stretchy for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <mrow>\
    <mo form="${parsedKey.form}" symmetric="false">${parsedKey.characters}</mo>\
    <mspace height="2em"></mspace>\
  </mrow>\
</math>\
 VS \
<math>\
  <mrow>\
    <mo form="${parsedKey.form}" symmetric="false" stretchy="${defaultValue}">${parsedKey.characters}</mo>\
    <mspace height="2em"></mspace>\
  </mrow>\
</math>\
</div>`);
            var div = document.body.lastElementChild;
            var mos = div.getElementsByTagName("mo");
            mo = mos[0].getBoundingClientRect()
            moRef = mos[1].getBoundingClientRect()
            assert_approx_equals(mo.height, moRef.height, epsilon, `Stretchy property for ${key} should be '${defaultValue}'`);
            div.style.display = "none";
        }
    },

    "symmetric": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        // FIXME: Should really detect symmetric support...
        assert_true(MathMLFeatureDetection.has_mspace());
        var defaultValue = defaultPropertyValue(entry, "symmetric");
        document.body.insertAdjacentHTML("beforeend", `<div>\
symmetric for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <mrow>\
    <mo form="${parsedKey.form}" stretchy="true">${parsedKey.characters}</mo>\
    <mspace height="1.5em"></mspace>\
  </mrow>\
</math>\
 VS \
<math>\
  <mrow>\
    <mo form="${parsedKey.form}" stretchy="true" symmetric="${defaultValue}">${parsedKey.characters}</mo>\
    <mspace height="1.5em"></mspace>\
  </mrow>\
</math>\
</div>`);
        var div = document.body.lastElementChild;
        var mos = div.getElementsByTagName("mo");
        mo = mos[0].getBoundingClientRect()
        moRef = mos[1].getBoundingClientRect()
        assert_approx_equals(mo.height, moRef.height, epsilon, `Symmetric property for ${key} should be '${defaultValue}'`);
        div.style.display = "none";
    },

    "accent": function(json, key) {
        let parsedKey = splitKey(key);
        let entry = json.dictionary[key];
        let epsilon = 1;

        // FIXME: Should really detect accent support...
        assert_true(MathMLFeatureDetection.has_mover());
        var defaultValue = defaultPropertyValue(entry, "accent");
        document.body.insertAdjacentHTML("beforeend", `<div>\
accent for "${parsedKey.characters}" (${parsedKey.form}): \
<math>\
  <mover>\
    <mn>&nbsp;</mn>\
    <mo form="${parsedKey.form}">${parsedKey.characters}</mo>\
  </mover>\
</math>\
 VS \
<math>\
  <mover>\
    <mn>&nbsp;</mn>\
    <mo form="${parsedKey.form}" accent="${defaultValue}">${parsedKey.characters}</mo>\
  </mover>\
</math>\
</div>`);
        var div = document.body.lastElementChild;
        var movers = div.getElementsByTagName("mover");
        function gapBetweenBaseAndScript(mover) {
            return mover.children[0].getBoundingClientRect().top -
                mover.children[1].getBoundingClientRect().bottom;
        }
        var gap = gapBetweenBaseAndScript(movers[0])
        var gapRef = gapBetweenBaseAndScript(movers[1])
        assert_approx_equals(gap, gapRef, epsilon, `Accent property for ${key} should be '${defaultValue}'`);
        div.style.display = "none";
    },

    run: function(json, name) {
        // The operator dictionary has more than one thousand of entries so the
        // tests are grouped in chunks so that these don't get much more
        // importance than other MathML tests. For easy debugging, one can set the
        // chunk size to 1. Also, note that the test div will remain visible for
        // failed tests.
        const entryPerChunk = 50

        var counter = 0;
        var test;

        for (key in json.dictionary) {

            if (counter % entryPerChunk === 0) {
                // Start of a new chunk.
                // Complete current async tests and create new ones for the next chunk.
                if (test) test.done();
                test = async_test(`Operator dictionary chunk ${1 + counter / entryPerChunk} - ${name}`);
            }

            test.step(function() {
                OperatorDictionaryTests[name](json, key);
            });

          counter++;
        }

        // Complete current async test.
        if (test) test.done();
    }
};
