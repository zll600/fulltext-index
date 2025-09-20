use crate::document::DocumentId;
use crate::index::InvertedIndex;
use std::collections::{HashMap, HashSet};

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
            BooleanOperator::And => result_sets
                .into_iter()
                .reduce(|acc, set| acc.intersection(&set).cloned().collect())
                .unwrap_or_default(),
            BooleanOperator::Or => result_sets
                .into_iter()
                .reduce(|acc, set| acc.union(&set).cloned().collect())
                .unwrap_or_default(),
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

    fn calculate_tfidf(
        &self,
        term_frequency: usize,
        document_frequency: usize,
        total_docs: usize,
    ) -> f64 {
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

    pub fn boolean_search(
        &self,
        operator: BooleanOperator,
        queries: Vec<&str>,
    ) -> Vec<SearchResult> {
        let query = Query::Boolean {
            operator,
            queries: queries
                .into_iter()
                .map(|q| Query::Term(q.to_string()))
                .collect(),
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

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    fn create_test_index() -> InvertedIndex {
        let mut index = InvertedIndex::new();

        index.add_document(
            "AI Research".to_string(),
            "artificial intelligence research methods".to_string(),
        );
        index.add_document(
            "Machine Learning".to_string(),
            "machine learning algorithms and techniques".to_string(),
        );
        index.add_document(
            "Deep Learning".to_string(),
            "deep learning neural networks".to_string(),
        );
        index.add_document(
            "Data Science".to_string(),
            "data science and machine learning applications".to_string(),
        );
        index.add_document(
            "Search Engines".to_string(),
            "search engine algorithms and information retrieval".to_string(),
        );

        index
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            doc_id: 1,
            score: 0.85,
            title: "Test Document".to_string(),
            snippet: "This is a test snippet".to_string(),
        };

        assert_eq!(result.doc_id, 1);
        assert_eq!(result.score, 0.85);
        assert_eq!(result.title, "Test Document");
        assert_eq!(result.snippet, "This is a test snippet");
    }

    #[test]
    fn test_query_enum() {
        // Test different query types
        let term_query = Query::Term("search".to_string());
        let boolean_query = Query::Boolean {
            operator: BooleanOperator::And,
            queries: vec![
                Query::Term("search".to_string()),
                Query::Term("engine".to_string()),
            ],
        };
        let phrase_query = Query::Phrase(vec!["machine".to_string(), "learning".to_string()]);
        let wildcard_query = Query::Wildcard("learn*".to_string());

        match term_query {
            Query::Term(term) => assert_eq!(term, "search"),
            _ => panic!("Expected Term query"),
        }

        match boolean_query {
            Query::Boolean { operator, queries } => {
                assert!(matches!(operator, BooleanOperator::And));
                assert_eq!(queries.len(), 2);
            }
            _ => panic!("Expected Boolean query"),
        }
        match phrase_query {
            Query::Phrase(terms) => {
                assert_eq!(terms.len(), 2);
                assert!(terms[0] == "machine" && terms[1] == "learning");
            }
            _ => panic!("Expected Phrase query"),
        }

        match wildcard_query {
            Query::Wildcard(pattern) => assert_eq!(pattern, "learn*"),
            _ => panic!("Expected Wildcard query"),
        }
    }

    #[test]
    fn test_searcher_creation() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        assert!(std::ptr::eq(searcher.index, &index));
    }

    #[test]
    fn test_simple_term_search() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        let results = searcher.search("machine");

        // Should find 2 documents containing "machine"
        assert_eq!(results.len(), 2);

        // Results should be sorted by score (highest first)
        assert!(results[0].score >= results[1].score);

        // All results should contain the search term in title or snippet
        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            assert!(text.contains("machine"));
        }
    }

    #[test]
    fn test_tfidf_scoring() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        // Search for a term that appears in multiple documents with different frequencies
        let results = searcher.search("learning");

        assert!(!results.is_empty());

        // All scores should be positive
        for result in &results {
            assert!(result.score > 0.0);
        }

        // Results should be sorted by score (descending)
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    fn test_boolean_and_search() {
        let index = create_test_index();
        let query = Query::Boolean {
            operator: BooleanOperator::And,
            queries: vec![
                Query::Term("machine".to_string()),
                Query::Term("learning".to_string()),
            ],
        };
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing both "machine" AND "learning"
        assert!(!results.is_empty());

        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            assert!(text.contains("machine") && text.contains("learning"));
        }
    }

    #[test]
    fn test_boolean_or_search() {
        let index = create_test_index();
        let query = Query::Boolean {
            operator: BooleanOperator::Or,
            queries: vec![
                Query::Term("artificial".to_string()),
                Query::Term("neural".to_string()),
            ],
        };
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing either "artificial" OR "neural"
        assert!(!results.is_empty());

        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            assert!(text.contains("artificial") || text.contains("neural"));
        }
    }

    #[test]
    fn test_boolean_not_search() {
        let index = create_test_index();
        let query = Query::Boolean {
            operator: BooleanOperator::Not,
            queries: vec![
                Query::Term("learning".to_string()),
                Query::Term("machine".to_string()),
            ],
        };
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing "learning" but NOT "machine"
        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            assert!(text.contains("learning") && !text.contains("machine"));
        }
    }

    #[test]
    fn test_phrase_search() {
        let index = create_test_index();
        let query = Query::Phrase(vec!["machine".to_string(), "learning".to_string()]);
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing the exact phrase "machine learning"
        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            assert!(text.contains("machine learning"));
        }
    }

    #[test]
    fn test_wildcard_prefix_search() {
        let index = create_test_index();
        let query = Query::Wildcard("learn*".to_string());
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing words starting with "learn"
        assert!(!results.is_empty());

        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            // Should match "learning"
            assert!(text.contains("learning"));
        }
    }

    #[test]
    fn test_wildcard_suffix_search() {
        let index = create_test_index();
        let query = Query::Wildcard("*ence".to_string());
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        // Should find documents containing words ending with "ence"
        for result in &results {
            let text = format!(
                "{} {}",
                result.title.to_lowercase(),
                result.snippet.to_lowercase()
            );
            // Should match "intelligence" or "science"
            assert!(text.contains("intelligence") || text.contains("science"));
        }
    }

    #[test]
    fn test_search_empty_query() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        let results = searcher.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nonexistent_term() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        let results = searcher.search("nonexistentterm");
        assert!(results.is_empty());
    }

    #[test]
    fn test_snippet_generation() {
        let mut index = InvertedIndex::new();
        let long_content = "This is a very long document content that contains many words and should be truncated when generating snippets for search results to ensure readability.";
        index.add_document("Long Document".to_string(), long_content.to_string());

        let searcher = Searcher::new(&index);
        let results = searcher.search("document");

        assert_eq!(results.len(), 1);
        let snippet = &results[0].snippet;

        // Snippet should be shorter than original content
        assert!(snippet.len() < long_content.len());

        // Snippet should contain the search term
        assert!(snippet.to_lowercase().contains("document"));
    }

    #[test]
    fn test_snippet_with_ellipsis() {
        let mut index = InvertedIndex::new();
        let content = "The beginning of this document is not very important but the middle contains the search term and the end also continues with more text.";
        index.add_document("Test Doc".to_string(), content.to_string());

        let searcher = Searcher::new(&index);
        let results = searcher.search("search");

        assert_eq!(results.len(), 1);
        let snippet = &results[0].snippet;

        // Should contain ellipsis if text is truncated
        if snippet.len() < content.len() {
            assert!(snippet.contains("..."));
        }
    }

    #[test]
    fn test_boolean_empty_queries() {
        let index = create_test_index();
        let query = Query::Boolean {
            operator: BooleanOperator::And,
            queries: vec![],
        };
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        assert!(results.is_empty());
    }

    #[test]
    fn test_phrase_search_empty() {
        let index = create_test_index();
        let query = Query::Phrase(vec![]);
        let searcher = Searcher::new(&index);
        let results = searcher.search_with_query(&query);

        assert!(results.is_empty());
    }

    #[test]
    fn test_case_insensitive_search() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        let lower_results = searcher.search("machine");
        let upper_results = searcher.search("MACHINE");
        let mixed_results = searcher.search("Machine");

        // All searches should return the same results
        assert_eq!(lower_results.len(), upper_results.len());
        assert_eq!(lower_results.len(), mixed_results.len());

        // Results should have the same document IDs
        let lower_ids: Vec<_> = lower_results.iter().map(|r| r.doc_id).collect();
        let upper_ids: Vec<_> = upper_results.iter().map(|r| r.doc_id).collect();
        let mixed_ids: Vec<_> = mixed_results.iter().map(|r| r.doc_id).collect();

        assert_eq!(lower_ids, upper_ids);
        assert_eq!(lower_ids, mixed_ids);
    }

    #[test]
    fn test_index_search_methods() {
        let index = create_test_index();

        // Test TF-IDF search
        let tfidf_results = index.search_tfidf("machine");
        assert!(!tfidf_results.is_empty());

        // Test boolean search
        let boolean_results =
            index.boolean_search(BooleanOperator::And, vec!["machine", "learning"]);
        assert!(!boolean_results.is_empty());

        // Test phrase search
        let phrase_results = index.phrase_search("machine learning");
        assert!(!phrase_results.is_empty());

        // Test wildcard search
        let wildcard_results = index.wildcard_search("learn*");
        assert!(!wildcard_results.is_empty());
    }

    #[test]
    fn test_result_deduplication_in_wildcard() {
        let mut index = InvertedIndex::new();
        index.add_document(
            "Learning Doc".to_string(),
            "machine learning and deep learning".to_string(),
        );

        let results = index.wildcard_search("learn*");

        // Even though "learning" appears twice, document should only appear once
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_tfidf_calculation() {
        let index = create_test_index();
        let searcher = Searcher::new(&index);

        // Test the TF-IDF calculation directly
        let score = searcher.calculate_tfidf(2, 1, 5); // tf=2, df=1, total_docs=5

        // Score should be positive
        assert!(score > 0.0);

        // Higher term frequency should give higher score
        let score_higher_tf = searcher.calculate_tfidf(3, 1, 5);
        assert!(score_higher_tf > score);

        // Lower document frequency should give higher score (more rare terms are more important)
        let score_lower_df = searcher.calculate_tfidf(2, 1, 5);
        let score_higher_df = searcher.calculate_tfidf(2, 3, 5);
        assert!(score_lower_df > score_higher_df);
    }
}
