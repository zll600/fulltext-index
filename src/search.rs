use std::collections::{HashMap, HashSet};
use crate::document::DocumentId;
use crate::index::InvertedIndex;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub doc_id: DocumentId,
    pub score: f64,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub enum BooleanOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum Query {
    Term(String),
    Boolean {
        operator: BooleanOperator,
        queries: Vec<Query>,
    },
    Phrase(Vec<String>),
    Wildcard(String),
}

pub struct Searcher<'a> {
    index: &'a InvertedIndex,
}

impl<'a> Searcher<'a> {
    pub fn new(index: &'a InvertedIndex) -> Self {
        Self { index }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query = Query::Term(query.to_string());
        self.execute_query(&query)
    }

    pub fn search_with_query(&self, query: &Query) -> Vec<SearchResult> {
        self.execute_query(query)
    }

    fn execute_query(&self, query: &Query) -> Vec<SearchResult> {
        match query {
            Query::Term(term) => self.search_term(term),
            Query::Boolean { operator, queries } => self.search_boolean(operator, queries),
            Query::Phrase(terms) => self.search_phrase(terms),
            Query::Wildcard(pattern) => self.search_wildcard(pattern),
        }
    }

    fn search_term(&self, term: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let normalized_term = term.to_lowercase();
        
        if let Some(posting_list) = self.index.get_posting_list(&normalized_term) {
            for posting in &posting_list.postings {
                let score = self.calculate_tfidf(
                    posting.term_frequency,
                    posting_list.document_frequency,
                    self.index.total_documents(),
                );
                
                if let Some(doc) = self.index.get_document(posting.doc_id) {
                    let snippet = self.generate_snippet(&doc.content, &normalized_term);
                    results.push(SearchResult {
                        doc_id: posting.doc_id,
                        score,
                        title: doc.title.clone(),
                        snippet,
                    });
                }
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    fn search_boolean(&self, operator: &BooleanOperator, queries: &[Query]) -> Vec<SearchResult> {
        if queries.is_empty() {
            return Vec::new();
        }
        
        let mut result_sets: Vec<HashSet<DocumentId>> = Vec::new();
        let mut all_results: HashMap<DocumentId, SearchResult> = HashMap::new();
        
        for query in queries {
            let results = self.execute_query(query);
            let doc_ids: HashSet<DocumentId> = results.iter().map(|r| r.doc_id).collect();
            
            for result in results {
                all_results.insert(result.doc_id, result);
            }
            
            result_sets.push(doc_ids);
        }
        
        let final_doc_ids = match operator {
            BooleanOperator::And => {
                result_sets.into_iter().reduce(|acc, set| {
                    acc.intersection(&set).cloned().collect()
                }).unwrap_or_default()
            }
            BooleanOperator::Or => {
                result_sets.into_iter().reduce(|acc, set| {
                    acc.union(&set).cloned().collect()
                }).unwrap_or_default()
            }
            BooleanOperator::Not => {
                if result_sets.len() != 2 {
                    return Vec::new();
                }
                let base = &result_sets[0];
                let exclude = &result_sets[1];
                base.difference(exclude).cloned().collect()
            }
        };
        
        let mut results: Vec<SearchResult> = final_doc_ids
            .into_iter()
            .filter_map(|doc_id| all_results.get(&doc_id).cloned())
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    fn search_phrase(&self, terms: &[String]) -> Vec<SearchResult> {
        if terms.is_empty() {
            return Vec::new();
        }
        
        let first_term = &terms[0].to_lowercase();
        let mut candidates = HashSet::new();
        
        if let Some(posting_list) = self.index.get_posting_list(first_term) {
            for posting in &posting_list.postings {
                candidates.insert(posting.doc_id);
            }
        }
        
        for term in &terms[1..] {
            let term = term.to_lowercase();
            let mut new_candidates = HashSet::new();
            
            if let Some(posting_list) = self.index.get_posting_list(&term) {
                for posting in &posting_list.postings {
                    if candidates.contains(&posting.doc_id) {
                        new_candidates.insert(posting.doc_id);
                    }
                }
            }
            
            candidates = new_candidates;
        }
        
        let mut results = Vec::new();
        for doc_id in candidates {
            if let Some(doc) = self.index.get_document(doc_id) {
                if self.contains_phrase(&doc.full_text(), terms) {
                    let score = 1.0;
                    let snippet = self.generate_snippet(&doc.content, &terms.join(" "));
                    results.push(SearchResult {
                        doc_id,
                        score,
                        title: doc.title.clone(),
                        snippet,
                    });
                }
            }
        }
        
        results
    }

    fn search_wildcard(&self, pattern: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let pattern_lower = pattern.to_lowercase();
        
        let prefix = pattern_lower.trim_end_matches('*');
        let suffix = pattern_lower.trim_start_matches('*');
        let is_prefix = pattern_lower.ends_with('*') && !pattern_lower.starts_with('*');
        let is_suffix = pattern_lower.starts_with('*') && !pattern_lower.ends_with('*');
        
        for term in self.index.index.keys() {
            let matches = if is_prefix {
                term.starts_with(prefix)
            } else if is_suffix {
                term.ends_with(suffix)
            } else {
                term.contains(&pattern_lower.replace('*', ""))
            };
            
            if matches {
                results.extend(self.search_term(term));
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.dedup_by_key(|r| r.doc_id);
        results
    }

    fn calculate_tfidf(&self, term_frequency: usize, document_frequency: usize, total_docs: usize) -> f64 {
        let tf = (term_frequency as f64).log10() + 1.0;
        let idf = ((total_docs as f64) / (document_frequency as f64)).log10();
        tf * idf
    }

    fn generate_snippet(&self, content: &str, query: &str) -> String {
        let lower_content = content.to_lowercase();
        let lower_query = query.to_lowercase();
        
        if let Some(pos) = lower_content.find(&lower_query) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&content[start..end]);
            if end < content.len() {
                snippet.push_str("...");
            }
            snippet
        } else {
            content.chars().take(100).collect::<String>() + "..."
        }
    }

    fn contains_phrase(&self, text: &str, terms: &[String]) -> bool {
        let text_lower = text.to_lowercase();
        let phrase_lower = terms.join(" ").to_lowercase();
        text_lower.contains(&phrase_lower)
    }
}

impl InvertedIndex {
    pub fn search_tfidf(&self, query: &str) -> Vec<SearchResult> {
        let searcher = Searcher::new(self);
        searcher.search(query)
    }

    pub fn boolean_search(&self, operator: BooleanOperator, queries: Vec<&str>) -> Vec<SearchResult> {
        let query = Query::Boolean {
            operator,
            queries: queries.into_iter().map(|q| Query::Term(q.to_string())).collect(),
        };
        let searcher = Searcher::new(self);
        searcher.search_with_query(&query)
    }

    pub fn phrase_search(&self, phrase: &str) -> Vec<SearchResult> {
        let terms: Vec<String> = phrase.split_whitespace().map(|s| s.to_string()).collect();
        let query = Query::Phrase(terms);
        let searcher = Searcher::new(self);
        searcher.search_with_query(&query)
    }

    pub fn wildcard_search(&self, pattern: &str) -> Vec<SearchResult> {
        let query = Query::Wildcard(pattern.to_string());
        let searcher = Searcher::new(self);
        searcher.search_with_query(&query)
    }
}