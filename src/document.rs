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