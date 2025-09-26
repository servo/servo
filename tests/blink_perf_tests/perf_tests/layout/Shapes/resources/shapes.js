(function() {
    var templateParagraph = null;
    var templateFloatingNode = null;
    var DEFAULT_SHAPE_OBJECT_COUNT = 100;

    function createParagraphNode() {
        if (!templateParagraph) {
            templateParagraph = document.createElement("p");
            templateParagraph.innerHTML = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam at turpis placerat sapien congue viverra nec sed felis.\
                Aenean aliquam, justo eu condimentum pharetra, arcu eros blandit metus, nec lacinia nisi orci vitae nunc.\
                Proin orci libero, accumsan non dignissim at, sodales in sapien. Curabitur dui nibh, venenatis vel tempus vel, accumsan nec velit.\
                Nam sit amet tempor lacus. Sed mollis dolor nibh, non tempus leo. Donec magna odio, commodo id porta in, aliquam mollis eros.\
                Pellentesque vulputate gravida ligula in elementum. Fusce lacinia massa justo, at porttitor orci.\
                Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Donec odio quam, pulvinar ut porttitor ac, tempor vitae ligula.\
                Cras aliquet sapien id sapien mollis nec pulvinar mauris adipiscing. Praesent porttitor consequat augue, sit amet mollis justo condimentum eu.\
                Etiam ut erat pellentesque orci congue interdum. Nulla eu eros mi.\
                Curabitur rutrum, lorem ac malesuada pellentesque, sapien risus consequat massa, eget pellentesque nunc nulla vel sem.";
                templateParagraph.className = "contentParagraph";
        }

        var paragraph = templateParagraph.cloneNode(true);
        return paragraph;
    }

    function createFloatingNode(properties) {
        if (!templateFloatingNode) {
            templateFloatingNode = document.createElement("div");
            templateFloatingNode.className = "floatingObject";
        }

        var float = templateFloatingNode.cloneNode(false);
        for (prop in properties) {
            float.style[prop] = properties[prop];
        }
        return float;
    }

    function addArticles(floatingObjects, paragraphCount) {
        for (var i = 0; i < paragraphCount; ++i) {
            floatingObjects.appendChild(createParagraphNode());
        }
    }

    function createFloatingObjects(properties, floatingObjectCount) {
        var testBox = document.createElement("div");
        for (var i = 0; i < floatingObjectCount; ++i) {
            testBox.appendChild(createFloatingNode(properties));
            testBox.appendChild(createParagraphNode())
        }
        testBox.className = "testBox";
        return testBox;
    }

    function applyFloating() {
        var floatingObjects = document.getElementsByClassName('floatingObject');
        for (i = 0; i < floatingObjects.length; ++i) {
            floatingObjects[i].style.cssFloat = 'left';
        }
    }

    function createShapeOutsideTest(properties, shapeObjectCount) {
        shapeObjectCount = shapeObjectCount || DEFAULT_SHAPE_OBJECT_COUNT;

        var floatingObjects = createFloatingObjects(properties, shapeObjectCount);
        document.body.appendChild(floatingObjects);
        return {
            description: "Testing shapes with " + properties['webkitShapeOutside'] +" using " + shapeObjectCount + " shapes.",
            run: function() {
                applyFloating();
                PerfTestRunner.forceLayout();
            },
            setup: function() {
                PerfTestRunner.resetRandomSeed();
                PerfTestRunner.forceLayout();
            },
            done: function() {
                document.body.removeChild(floatingObjects);
                templateParagraph = null;
            }
        };
    }

    window.createShapeOutsideTest = createShapeOutsideTest;

})();
