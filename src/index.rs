use std::collections::{HashMap, HashSet};
use crate::document::{Document, DocumentId, DocumentStore};

#[derive(Debug, Clone)]
pub struct TermPosition {
    pub position: usize,
    pub field: String,
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
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            document_store: DocumentStore::new(),
            total_terms: 0,
        }
    }

    pub fn add_document(&mut self, title: String, content: String) -> DocumentId {
        let doc_id = self.document_store.add_document(title.clone(), content.clone());
        let document = self.document_store.get_document(doc_id).unwrap();
        
        let title_terms = self.extract_terms(&document.title, "title");
        let content_terms = self.extract_terms(&document.content, "content");
        
        let mut term_positions: HashMap<String, Vec<TermPosition>> = HashMap::new();
        
        for (term, positions) in title_terms {
            term_positions.entry(term).or_insert_with(Vec::new).extend(positions);
        }
        
        for (term, positions) in content_terms {
            term_positions.entry(term).or_insert_with(Vec::new).extend(positions);
        }
        
        for (term, positions) in term_positions {
            let posting_list = self.index.entry(term.clone()).or_insert_with(|| PostingList::new(term));
            posting_list.add_posting(doc_id, positions);
            self.total_terms += 1;
        }
        
        doc_id
    }

    fn extract_terms(&self, text: &str, field: &str) -> HashMap<String, Vec<TermPosition>> {
        let mut terms = HashMap::new();
        let tokens: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        for (position, token) in tokens.iter().enumerate() {
            let term_position = TermPosition {
                position,
                field: field.to_string(),
            };
            terms.entry(token.clone())
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
            posting_list.postings
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