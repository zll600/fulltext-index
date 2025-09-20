use crate::document::{Document, DocumentId, DocumentStore};
use crate::tokenizer::Tokenizer;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Title,
    Content,
}

#[derive(Debug, Clone)]
pub struct TermPosition {
    pub position: usize,
    pub field: FieldType,
}

#[derive(Debug)]
pub struct PostingEntry {
    pub doc_id: DocumentId,
    pub term_frequency: usize,
    pub positions: Vec<TermPosition>,
}

#[derive(Debug)]
pub struct PostingList {
    pub term: String,
    pub document_frequency: usize,
    pub postings: Vec<PostingEntry>,
}

impl PostingList {
    fn new(term: String) -> Self {
        Self {
            term,
            document_frequency: 0,
            postings: Vec::new(),
        }
    }

    fn add_posting(&mut self, doc_id: DocumentId, positions: Vec<TermPosition>) {
        let term_frequency = positions.len();
        self.postings.push(PostingEntry {
            doc_id,
            term_frequency,
            positions,
        });
        self.document_frequency += 1;
    }
}

pub struct InvertedIndex {
    pub index: HashMap<String, PostingList>,
    document_store: DocumentStore,
    total_terms: usize,
    tokenizer: Tokenizer,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            document_store: DocumentStore::new(),
            total_terms: 0,
            tokenizer: Tokenizer::new(),
        }
    }

    pub fn add_document(&mut self, title: String, content: String) -> DocumentId {
        let doc_id = self
            .document_store
            .add_document(title.clone(), content.clone());
        let document = self.document_store.get_document(doc_id).unwrap();

        let title_terms = self.extract_terms(&document.title, FieldType::Title);
        let content_terms = self.extract_terms(&document.content, FieldType::Content);

        let mut term_positions: HashMap<String, Vec<TermPosition>> = HashMap::new();

        for (term, positions) in title_terms {
            term_positions
                .entry(term)
                .or_insert_with(Vec::new)
                .extend(positions);
        }

        for (term, positions) in content_terms {
            term_positions
                .entry(term)
                .or_insert_with(Vec::new)
                .extend(positions);
        }

        for (term, positions) in term_positions {
            let posting_list = self
                .index
                .entry(term.clone())
                .or_insert_with(|| PostingList::new(term));
            posting_list.add_posting(doc_id, positions);
            self.total_terms += 1;
        }

        doc_id
    }

    fn extract_terms(&self, text: &str, field: FieldType) -> HashMap<String, Vec<TermPosition>> {
        let mut terms = HashMap::new();
        let tokens = self.tokenizer.tokenize(text);

        for token in tokens {
            let term_position = TermPosition {
                position: token.position,
                field: field.clone(),
            };
            terms
                .entry(token.text)
                .or_insert_with(Vec::new)
                .push(term_position);
        }

        terms
    }

    pub fn search(&self, query: &str) -> Vec<DocumentId> {
        let query_term = query.to_lowercase();

        if let Some(posting_list) = self.index.get(&query_term) {
            posting_list.postings.iter().map(|p| p.doc_id).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_document(&self, id: DocumentId) -> Option<&Document> {
        self.document_store.get_document(id)
    }

    pub fn total_documents(&self) -> usize {
        self.document_store.total_documents()
    }

    pub fn total_unique_terms(&self) -> usize {
        self.index.len()
    }

    pub fn get_posting_list(&self, term: &str) -> Option<&PostingList> {
        self.index.get(&term.to_lowercase())
    }

    pub fn get_term_frequency(&self, term: &str, doc_id: DocumentId) -> usize {
        if let Some(posting_list) = self.get_posting_list(term) {
            posting_list
                .postings
                .iter()
                .find(|p| p.doc_id == doc_id)
                .map(|p| p.term_frequency)
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub fn get_document_frequency(&self, term: &str) -> usize {
        self.get_posting_list(term)
            .map(|p| p.document_frequency)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_position_creation() {
        let pos = TermPosition {
            position: 5,
            field: FieldType::Title,
        };

        assert_eq!(pos.position, 5);
        assert_eq!(pos.field, FieldType::Title);
    }

    #[test]
    fn test_posting_list_creation() {
        let posting_list = PostingList::new("search".to_string());

        assert_eq!(posting_list.term, "search");
        assert_eq!(posting_list.document_frequency, 0);
        assert_eq!(posting_list.postings.len(), 0);
    }

    #[test]
    fn test_posting_list_add_posting() {
        let mut posting_list = PostingList::new("test".to_string());
        let positions = vec![
            TermPosition {
                position: 0,
                field: FieldType::Title,
            },
            TermPosition {
                position: 3,
                field: FieldType::Content,
            },
        ];

        posting_list.add_posting(1, positions);

        assert_eq!(posting_list.document_frequency, 1);
        assert_eq!(posting_list.postings.len(), 1);

        let posting = &posting_list.postings[0];
        assert_eq!(posting.doc_id, 1);
        assert_eq!(posting.term_frequency, 2);
        assert_eq!(posting.positions.len(), 2);
    }

    #[test]
    fn test_inverted_index_creation() {
        let index = InvertedIndex::new();

        assert_eq!(index.total_documents(), 0);
        assert_eq!(index.total_unique_terms(), 0);
        assert_eq!(index.total_terms, 0);
    }

    #[test]
    fn test_inverted_index_add_document() {
        let mut index = InvertedIndex::new();
        const TITLE: &str = "Machine Learning";
        const CONTENT: &str = "Machine learning is a subset of artificial intelligence";

        let doc_id = index.add_document(TITLE.to_string(), CONTENT.to_string());

        assert_eq!(doc_id, 0);
        assert_eq!(index.total_documents(), 1);
        assert!(index.total_unique_terms() > 0);

        let doc = index.get_document(doc_id).unwrap();
        assert_eq!(doc.title, TITLE);
        assert_eq!(doc.content, CONTENT);
    }

    #[test]
    fn test_inverted_index_term_extraction() {
        let mut index = InvertedIndex::new();

        index.add_document(
            "Simple Test".to_string(),
            "This is a simple test document".to_string(),
        );

        // Check that terms were indexed (case-insensitive)
        assert!(index.get_posting_list("simple").is_some());
        assert!(index.get_posting_list("test").is_some());
        assert!(index.get_posting_list("document").is_some());

        // Check that case normalization works - both cases should find the same terms
        assert!(index.get_posting_list("Simple").is_some()); // Normalized to "simple"
        assert!(index.get_posting_list("Test").is_some()); // Normalized to "test"
    }

    #[test]
    fn test_inverted_index_term_positions() {
        let mut index = InvertedIndex::new();

        index.add_document(
            "Test Document".to_string(),
            "This test contains the word test again".to_string(),
        );

        let posting_list = index.get_posting_list("test").unwrap();
        assert_eq!(posting_list.document_frequency, 1);

        let posting = &posting_list.postings[0];
        // "test" appears in title (position 0) and content (positions 1 and 5)
        assert_eq!(posting.term_frequency, 3);
        assert_eq!(posting.positions.len(), 3);

        // Check field types
        let title_positions: Vec<_> = posting
            .positions
            .iter()
            .filter(|p| p.field == FieldType::Title)
            .collect();
        let content_positions: Vec<_> = posting
            .positions
            .iter()
            .filter(|p| p.field == FieldType::Content)
            .collect();

        assert_eq!(title_positions.len(), 1);
        assert_eq!(content_positions.len(), 2);
    }

    #[test]
    fn test_inverted_index_multiple_documents() {
        let mut index = InvertedIndex::new();

        let _doc1 = index.add_document("First Doc".to_string(), "search engine".to_string());
        let _doc2 = index.add_document("Second Doc".to_string(), "search algorithm".to_string());
        let _doc3 = index.add_document("Third Doc".to_string(), "sorting algorithm".to_string());

        assert_eq!(index.total_documents(), 3);

        // "search" appears in 2 documents
        let search_posting = index.get_posting_list("search").unwrap();
        assert_eq!(search_posting.document_frequency, 2);
        assert_eq!(search_posting.postings.len(), 2);

        // "algorithm" appears in 2 documents
        let algorithm_posting = index.get_posting_list("algorithm").unwrap();
        assert_eq!(algorithm_posting.document_frequency, 2);
        assert_eq!(algorithm_posting.postings.len(), 2);

        // "engine" appears in 1 document
        let engine_posting = index.get_posting_list("engine").unwrap();
        assert_eq!(engine_posting.document_frequency, 1);
        assert_eq!(engine_posting.postings.len(), 1);
    }

    #[test]
    fn test_inverted_index_search() {
        let mut index = InvertedIndex::new();

        let doc1 = index.add_document(
            "AI Research".to_string(),
            "artificial intelligence research".to_string(),
        );
        let doc2 = index.add_document(
            "ML Basics".to_string(),
            "machine learning fundamentals".to_string(),
        );
        let doc3 = index.add_document(
            "AI Applications".to_string(),
            "artificial intelligence in practice".to_string(),
        );

        // Search for "artificial" should return docs 1 and 3
        let results = index.search("artificial");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&doc1));
        assert!(results.contains(&doc3));

        // Search for "machine" should return only doc 2
        let results = index.search("machine");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&doc2));

        // Search for non-existent term
        let results = index.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_inverted_index_term_frequency() {
        let mut index = InvertedIndex::new();

        let doc_id = index.add_document(
            "Repeated Words".to_string(),
            "search search search query".to_string(),
        );

        // "search" appears 3 times in content
        let tf = index.get_term_frequency("search", doc_id);
        assert_eq!(tf, 3);

        // "query" appears 1 time
        let tf = index.get_term_frequency("query", doc_id);
        assert_eq!(tf, 1);

        // Non-existent term
        let tf = index.get_term_frequency("nonexistent", doc_id);
        assert_eq!(tf, 0);
    }

    #[test]
    fn test_inverted_index_document_frequency() {
        let mut index = InvertedIndex::new();

        index.add_document("Doc 1".to_string(), "machine learning".to_string());
        index.add_document("Doc 2".to_string(), "machine intelligence".to_string());
        index.add_document("Doc 3".to_string(), "deep learning".to_string());

        // "machine" appears in 2 documents
        assert_eq!(index.get_document_frequency("machine"), 2);

        // "learning" appears in 2 documents
        assert_eq!(index.get_document_frequency("learning"), 2);

        // "intelligence" appears in 1 document
        assert_eq!(index.get_document_frequency("intelligence"), 1);

        // Non-existent term
        assert_eq!(index.get_document_frequency("nonexistent"), 0);
    }

    #[test]
    fn test_inverted_index_punctuation_handling() {
        let mut index = InvertedIndex::new();

        index.add_document(
            "Punctuation Test".to_string(),
            "Hello, world! This is a test... with punctuation?".to_string(),
        );

        // Punctuation should be stripped
        assert!(index.get_posting_list("hello").is_some());
        assert!(index.get_posting_list("world").is_some());
        assert!(index.get_posting_list("punctuation").is_some());

        // Punctuation marks should not be indexed
        assert!(index.get_posting_list(",").is_none());
        assert!(index.get_posting_list("!").is_none());
        assert!(index.get_posting_list("?").is_none());
    }

    #[test]
    fn test_inverted_index_empty_content() {
        let mut index = InvertedIndex::new();

        let doc_id = index.add_document("".to_string(), "".to_string());

        assert_eq!(index.total_documents(), 1);
        assert_eq!(index.total_unique_terms(), 0);

        let doc = index.get_document(doc_id).unwrap();
        assert_eq!(doc.title, "");
        assert_eq!(doc.content, "");
    }

    #[test]
    fn test_inverted_index_whitespace_only() {
        let mut index = InvertedIndex::new();

        index.add_document("   ".to_string(), "   \n\t  ".to_string());

        assert_eq!(index.total_documents(), 1);
        assert_eq!(index.total_unique_terms(), 0);
    }

    #[test]
    fn test_tokenizer_integration_stop_words() {
        let mut index = InvertedIndex::new();

        // Add a document with stop words
        index.add_document(
            "Test Document".to_string(),
            "The quick brown fox is an animal".to_string(),
        );

        // Stop words should NOT be indexed
        assert!(index.get_posting_list("the").is_none());
        assert!(index.get_posting_list("is").is_none());
        assert!(index.get_posting_list("an").is_none());

        // Content words should be indexed
        assert!(index.get_posting_list("test").is_some());
        assert!(index.get_posting_list("document").is_some());
        assert!(index.get_posting_list("quick").is_some());
        assert!(index.get_posting_list("brown").is_some());
        assert!(index.get_posting_list("fox").is_some());
        assert!(index.get_posting_list("animal").is_some());

        // Should have 6 unique terms (test, document, quick, brown, fox, animal)
        assert_eq!(index.total_unique_terms(), 6);
    }

    #[test]
    fn test_tokenizer_integration_min_length() {
        let mut index = InvertedIndex::new();

        // Add document with short words
        index.add_document("Short Words".to_string(), "I a go to it".to_string());

        // Single character words should not be indexed (min length is 2)
        assert!(index.get_posting_list("i").is_none());
        assert!(index.get_posting_list("a").is_none());

        // Two character words should be indexed (unless they're stop words)
        assert!(index.get_posting_list("go").is_some());
        // "to" and "it" are stop words, so they won't be indexed
        assert!(index.get_posting_list("to").is_none());
        assert!(index.get_posting_list("it").is_none());

        // From title
        assert!(index.get_posting_list("short").is_some());
        assert!(index.get_posting_list("words").is_some());
    }
}
