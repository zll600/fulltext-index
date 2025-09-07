use std::collections::HashMap;

pub type DocumentId = usize;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

impl Document {
    pub fn new(id: DocumentId, title: String, content: String) -> Self {
        Self {
            id,
            title,
            content,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn full_text(&self) -> String {
        format!("{} {}", self.title, self.content)
    }
}

#[derive(Debug)]
pub struct DocumentStore {
    documents: HashMap<DocumentId, Document>,
    next_id: DocumentId,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_document(&mut self, title: String, content: String) -> DocumentId {
        let id = self.next_id;
        self.next_id += 1;
        let doc = Document::new(id, title, content);
        self.documents.insert(id, doc);
        id
    }

    pub fn get_document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    pub fn total_documents(&self) -> usize {
        self.documents.len()
    }

    pub fn all_documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        const TITLE: &str = "Test Title";
        const CONTENT: &str = "Test content";
        let doc = Document::new(42, TITLE.to_string(), CONTENT.to_string());
        
        assert_eq!(doc.id, 42);
        assert_eq!(doc.title, TITLE);
        assert_eq!(doc.content, CONTENT);
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn test_document_with_metadata() {
        let doc = Document::new(1, "Title".to_string(), "Content".to_string())
            .with_metadata("author".to_string(), "John Doe".to_string())
            .with_metadata("category".to_string(), "Tech".to_string());
        
        assert_eq!(doc.metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(doc.metadata.get("category"), Some(&"Tech".to_string()));
        assert_eq!(doc.metadata.len(), 2);
    }

    #[test]
    fn test_document_full_text() {
        let doc = Document::new(1, "Hello World".to_string(), "This is content".to_string());
        
        assert_eq!(doc.full_text(), "Hello World This is content");
    }

    #[test]
    fn test_document_store_creation() {
        let store = DocumentStore::new();
        
        assert_eq!(store.total_documents(), 0);
        assert!(store.all_documents().next().is_none());
    }

    #[test]
    fn test_document_store_add_document() {
        let mut store = DocumentStore::new();
        
        let id1 = store.add_document("First Document".to_string(), "First content".to_string());
        let id2 = store.add_document("Second Document".to_string(), "Second content".to_string());
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(store.total_documents(), 2);
    }

    #[test]
    fn test_document_store_get_document() {
        let mut store = DocumentStore::new();
        const TITLE: &str = "Test Doc";
        const CONTENT: &str = "Test content";
        let id = store.add_document(TITLE.to_string(), CONTENT.to_string());
        
        let doc = store.get_document(id).unwrap();
        assert_eq!(doc.id, id);
        assert_eq!(doc.title, TITLE);
        assert_eq!(doc.content, CONTENT);
        
        // Test non-existent document
        assert!(store.get_document(999).is_none());
    }

    #[test]
    fn test_document_store_iteration() {
        let mut store = DocumentStore::new();
        store.add_document("Doc 1".to_string(), "Content 1".to_string());
        store.add_document("Doc 2".to_string(), "Content 2".to_string());
        store.add_document("Doc 3".to_string(), "Content 3".to_string());
        
        let documents: Vec<&Document> = store.all_documents().collect();
        assert_eq!(documents.len(), 3);
        
        // Check that all documents are present (order may vary due to HashMap)
        let titles: Vec<&String> = documents.iter().map(|d| &d.title).collect();
        assert!(titles.contains(&&"Doc 1".to_string()));
        assert!(titles.contains(&&"Doc 2".to_string()));
        assert!(titles.contains(&&"Doc 3".to_string()));
    }

    #[test]
    fn test_document_store_sequential_ids() {
        let mut store = DocumentStore::new();
        
        let ids: Vec<DocumentId> = (0..5)
            .map(|i| store.add_document(format!("Doc {}", i), format!("Content {}", i)))
            .collect();
        
        // IDs should be sequential starting from 0
        assert_eq!(ids, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_document_empty_title_and_content() {
        let doc = Document::new(1, "".to_string(), "".to_string());
        
        assert_eq!(doc.title, "");
        assert_eq!(doc.content, "");
        assert_eq!(doc.full_text(), " ");  // Title + space + content
    }
}