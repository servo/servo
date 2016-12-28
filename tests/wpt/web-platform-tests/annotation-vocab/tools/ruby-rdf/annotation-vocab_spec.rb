require "bundler/setup"
require 'rdf'
require 'linkeddata'
require 'rdf/spec/matchers'
require 'rspec'

SAMPLES = File.expand_path("../../samples/", __FILE__)
CORRECT = Dir.glob(File.join(SAMPLES, "correct/*.json"))
INCORRECT = Dir.glob(File.join(SAMPLES, "incorrect/*.json"))
RDF::Reasoner.apply(:rdfs, :owl)
VOCAB_URI = "http://www.w3.org/ns/oa#"
VOCAB_GRAPH = begin
  g = RDF::Graph.load(VOCAB_URI, format: :jsonld, headers: {"Accept" => "application/ld+json"})
  g.each_object {|o| o.squish! if o.literal?}
  g
end

describe "Web Annotation Vocab" do
  let(:vocab) {VOCAB_URI}
  let(:vocab_graph) {VOCAB_GRAPH}

  # Load Annotation vocabulary first, so that that defined in rdf-vocab is not lazy-loaded
  before(:all) do
    RDF::Vocabulary.from_graph(VOCAB_GRAPH, url: VOCAB_URI, class_name: RDF::Vocab::OA)
  end

  it "The JSON-LD context document can be parsed without errors by JSON-LD validators" do
    expect {JSON::LD::API.expand(vocab, validate: true)}.not_to raise_error
  end

  context "The JSON-LD context document can be used to convert JSON-LD serialized Annotations into RDF triples" do
    CORRECT.each do |file|
      it "#{file.split('/').last}" do
        nt = file.sub(/.json$/, '.nt')
        gjld = RDF::Graph.load(file, format: :jsonld)
        gnt = RDF::Graph.load(nt, format: :ntriples)
        expect(gjld).to be_equivalent_graph(gnt)
      end

      it "lint #{file.split('/').last}" do
        gjld = RDF::Graph.load(file, format: :jsonld)
        gjld.entail!
        expect(gjld.lint).to be_empty
      end
    end
  end

  context "detects errors in incorrect examples" do
    INCORRECT.each do |file|
      it "#{file.split('/').last}" do
        pending "Empty Documents are invalid" if file =~ /anno2.json|anno3.json/
        expect {RDF::Graph.load(file, validate: true, format: :jsonld, logger: false)}.to raise_error(RDF::ReaderError)
      end
    end
  end

  context "The ontology documents can be parsed without errors by RDF Schema validators" do
    {
      jsonld: "application/ld+json",
      rdfxml: "application/rdf+xml",
      ttl: "text/turtle",
    }.each do |format, content_type|
      it "JSON-LD version is isomorphic to #{format}" do
        expect do
          RDF::Graph.load(vocab, format: format, validate: true, headers: {"Accept" => content_type})
        end.not_to raise_error
      end
    end
  end

  context "The ontology documents are isomorphic to each other" do
    {
      rdfxml: "application/rdf+xml",
      ttl: "text/turtle",
    }.each do |format, content_type|
      it format do
        fg = RDF::Graph.load(vocab, format: format, headers: {"Accept" => content_type})

        # XXX Normalize whitespace in literals to ease comparision
        fg.each_object {|o| o.squish! if o.literal?}
        expect(fg).to be_equivalent_graph(vocab_graph)
      end
    end
  end

  context "The ontology is internally consistent with respect to domains, ranges, inverses, and any other ontology features specified." do
    it "lints cleanly" do
      entailed_graph = vocab_graph.dup.entail!
      expect(entailed_graph.lint).to be_empty
    end

    RDF::Vocab::OA.each do |term|
      if term.type.to_s =~ /Class/
        context term.pname do
          it "subClassOf" do
            expect {term.subClassOf.map(&:pname)}.not_to raise_error
          end
          it "equivalentClass" do
            expect {term.equivalentClass.map(&:pname)}.not_to raise_error
          end
        end
      elsif term.type.to_s =~ /Property/
        context term.pname do
          it "subPropertyOf" do
            expect {term.subPropertyOf.map(&:pname)}.not_to raise_error
          end
          it "domain" do
            expect {term.domain.map(&:pname)}.not_to raise_error
          end
          it "range" do
            expect {term.range.map(&:pname)}.not_to raise_error
          end
          it "equivalentProperty" do
            expect {term.equivalentProperty.map(&:pname)}.not_to raise_error
          end
        end
      end
    end
  end
end
