
# Author: Rob Sanderson (azaroth42@gmail.com)
# License: Apache2
# Last Modified: 2016-09-02

import json
from rdflib import ConjunctiveGraph, URIRef
from pyld import jsonld
from pyld.jsonld import compact, expand, frame, from_rdf, to_rdf, JsonLdProcessor
import urllib

# Stop code from looking up the contexts online for every operation
docCache = {}

def fetch(url):
    fh = urllib.urlopen(url)
    data = fh.read()
    fh.close()
    return data

def load_document_and_cache(url):
    if docCache.has_key(url):
        return docCache[url]

    doc = {
        'contextUrl': None,
        'documentUrl': None,
        'document': ''
    }
    data = fetch(url)
    doc['document'] = data;
    docCache[url] = doc
    return doc

jsonld.set_document_loader(load_document_and_cache)

class Validator(object):

    def __init__(self):

        self.rdflib_class_map = {
            "Annotation":           "oa:Annotation",
            "Dataset":              "dctypes:Dataset",
            "Image":                "dctypes:StillImage",
            "Video":                "dctypes:MovingImage",
            "Audio":                "dctypes:Sound",
            "Text":                 "dctypes:Text",
            "TextualBody":          "oa:TextualBody",
            "ResourceSelection":    "oa:ResourceSelection",
            "SpecificResource":     "oa:SpecificResource",
            "FragmentSelector":     "oa:FragmentSelector",
            "CssSelector":          "oa:CssSelector",
            "XPathSelector":        "oa:XPathSelector",
            "TextQuoteSelector":    "oa:TextQuoteSelector",
            "TextPositionSelector": "oa:TextPositionSelector",
            "DataPositionSelector": "oa:DataPositionSelector",
            "SvgSelector":          "oa:SvgSelector",
            "RangeSelector":        "oa:RangeSelector",
            "TimeState":            "oa:TimeState",
            "HttpState":            "oa:HttpRequestState",
            "CssStylesheet":        "oa:CssStyle",
            "Choice":               "oa:Choice",
            "Composite":            "oa:Composite",
            "List":                 "oa:List",
            "Independents":         "oa:Independents",
            "Person":               "foaf:Person",
            "Software":             "as:Application",
            "Organization":         "foaf:Organization",
            "AnnotationCollection": "as:OrderedCollection",
            "AnnotationPage":       "as:OrderedCollectionPage",
            "Audience":             "schema:Audience"
        }


    def _clean_bnode_ids(self, js):
        new = {}
        for (k,v) in js.items():
            if k == 'id' and v.startswith("_:"):
                continue
            elif type(v) == dict:
                # recurse
                res = self._clean_bnode_ids(v)
                new[k] = res
            else:
                new[k] = v
        return new

    def _mk_rdflib_jsonld(self, js):
        # rdflib's json-ld implementation sucks
        # Pre-process to make it work
        # recurse the structure looking for types, and replacing them.
        new = {}
        for (k,v) in js.items():
            if k == 'type':
                if type(v) == list:
                    nl = []
                    for i in v:
                        if self.rdflib_class_map.has_key(i):
                            nl.append(self.rdflib_class_map[i])
                    new['type'] = nl
                else:
                    if self.rdflib_class_map.has_key(v):
                        new['type'] = self.rdflib_class_map[v]
            elif type(v) == dict:
                # recurse
                res = self._mk_rdflib_jsonld(v)
                new[k] = res
            else:
                new[k] = v
        return new

    def json_to_rdf(self, js, fmt=None):
        d2 = self._mk_rdflib_jsonld(js)
        js = json.dumps(d2)
        g = ConjunctiveGraph()
        g.parse(data=js, format='json-ld')
        if fmt:
            out = g.serialize(format=fmt)
            return out
        else:
            return g

    def rdf_to_jsonld(self, rdf, fmt):

        g = ConjunctiveGraph()
        g.parse(data=rdf, format=fmt)
        out = g.serialize(format='json-ld')

        j2 = json.loads(out)
        j2 = {"@context": context_js, "@graph": j2}
        framed = frame(j2, frame_js)
        out = compact(framed, context_js)
        # recursively clean blank node ids
        #out = self._clean_bnode_ids(out)
        return out

    def compact_and_clean(self, js):
        newjs = compact(js, context_js)
        newjs['@context'] = context
        if newjs.has_key("@graph"):
            for k,v in newjs['@graph'].items():
                newjs[k] = v
            del newjs['@graph']
        return newjs

validator = Validator()

example = "https://raw.githubusercontent.com/w3c/web-annotation/gh-pages/model/wd2/examples/correct/anno4.json"
example_ttl = "https://raw.githubusercontent.com/w3c/web-annotation/gh-pages/vocab/wd/examples/correct/anno1.ttl"
context = "http://www.w3.org/ns/anno.jsonld"
frameURI = "https://raw.githubusercontent.com/w3c/web-annotation/gh-pages/jsonld/annotation_frame.jsonld"
# ontology = "https://www.w3.org/ns/oa.ttl"
ontology = "https://raw.githubusercontent.com/w3c/web-annotation/gh-pages/vocab/wd/ontology/oa.ttl"

data = fetch(context)
context_js = json.loads(data)
data = fetch(example)
example_js = json.loads(data)
data = fetch(frameURI)
frame_js = json.loads(data)

# Test1:  JSON-LD context document can be parsed without errors by JSON-LD validators
# Context document is parsable if it can be loaded and used to expand the example
try:
    expanded = expand(example_js, context_js)
except:
    print "Context is invalid, failed Test 1"


# Test2: JSON-LD context document can be used to convert JSON-LD serialized Annotations into RDF triples.
try:
    jsonld_nq = to_rdf(example_js, {"base": "http://example.org/", "format": "application/nquads"})
except:
    print "Cannot use context to convert JSON-LD to NQuads"


# Test3: Graphs produced are isomorphic
try:
    rl_g = validator.json_to_rdf(example_js)
    g = ConjunctiveGraph()
    js_g = g.parse(data=jsonld_nq, format="nt")
    rl_g_nq = rl_g.serialize(format="nquads")
    assert(len(rl_g.store) == len(js_g.store))
    assert(rl_g.isomorphic(js_g))
except:
    print "Different triples from two parsers, or non-isomorphic graphs"


# Test4: The graphs produced can be converted back into JSON-LD without loss of information
try:
    js = validator.rdf_to_jsonld(jsonld_nq, "nt")
    js2 = validator.compact_and_clean(js)
    assert(js2 == example_js)
except:
    print "Failed to recompact parsed data"
    raise


# Test5: ontology documents can be parsed without errors by validators
try:
    g = ConjunctiveGraph().parse(ontology, format="turtle")
except:
    raise


# Test6: ontology is internally consistent with respect to domains, ranges, etc

# step 1: find all the classes.
rdftype = URIRef("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
rdfsdomain = URIRef("http://www.w3.org/2000/01/rdf-schema#domain")
rdfsrange = URIRef("http://www.w3.org/2000/01/rdf-schema#range")
rdfsresource = URIRef("http://www.w3.org/1999/02/22-rdf-syntax-ns#Resource")
rdfssco = URIRef("http://www.w3.org/2000/01/rdf-schema#subClassOf")
asColl = URIRef("http://www.w3.org/ns/activitystreams#OrderedCollection")
skosConcept = URIRef("http://www.w3.org/2004/02/skos/core#Concept")

otherClasses = [asColl, skosConcept]
classes = list(g.subjects(rdftype, URIRef("http://www.w3.org/2000/01/rdf-schema#Class")))
props = list(g.subjects(rdftype, URIRef("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property")))

for p in props:
    domains = list(g.objects(p, rdfsdomain))
    for d in domains:
        assert(d in classes)

for p in props:
    ranges = list(g.objects(p, rdfsrange))
    for r in ranges:
        if not r in classes and not str(r).startswith("http://www.w3.org/2001/XMLSchema#") and \
            not r == rdfsresource:
            print "Found inconsistent property: %s has unknown range" % p

for c in classes:
    parents = list(g.objects(c, rdfssco))
    for p in parents:
        if not p in classes and not p in otherClasses:
            print "Found inconsistent class: %s has unknown superClass" % c


print "Done."
