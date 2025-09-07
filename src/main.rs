use fulltext_index::{InvertedIndex, Document};
use fulltext_index::search::BooleanOperator;

fn main() {
    println!("=== Fulltext Index Demo ===\n");
    
    let mut index = InvertedIndex::new();
    
    println!("Adding documents to the index...");
    
    let doc1 = index.add_document(
        "Introduction to Information Retrieval".to_string(),
        "Information retrieval is the process of obtaining information system resources that are relevant to an information need from a collection of those resources.".to_string()
    );
    
    let doc2 = index.add_document(
        "Search Engine Technology".to_string(),
        "Modern search engines use inverted indexes to quickly find documents containing specific terms. The inverted index maps terms to document identifiers.".to_string()
    );
    
    let doc3 = index.add_document(
        "Natural Language Processing".to_string(),
        "Natural language processing enables computers to understand, interpret, and generate human language in a valuable way.".to_string()
    );
    
    let doc4 = index.add_document(
        "Database Systems".to_string(),
        "Database systems provide efficient storage and retrieval of structured data. They use indexes to speed up query processing.".to_string()
    );
    
    let doc5 = index.add_document(
        "Machine Learning Basics".to_string(),
        "Machine learning algorithms learn patterns from data to make predictions. Search engines use machine learning to improve ranking.".to_string()
    );
    
    println!("Index statistics:");
    println!("  Total documents: {}", index.total_documents());
    println!("  Total unique terms: {}", index.total_unique_terms());
    println!();
    
    println!("=== Search Examples ===\n");
    
    println!("1. Simple term search for 'search':");
    let results = index.search_tfidf("search");
    print_results(&index, &results);
    
    println!("\n2. Simple term search for 'language':");
    let results = index.search_tfidf("language");
    print_results(&index, &results);
    
    println!("\n3. Boolean AND search for 'search' AND 'engine':");
    let results = index.boolean_search(BooleanOperator::And, vec!["search", "engine"]);
    print_results(&index, &results);
    
    println!("\n4. Boolean OR search for 'database' OR 'retrieval':");
    let results = index.boolean_search(BooleanOperator::Or, vec!["database", "retrieval"]);
    print_results(&index, &results);
    
    println!("\n5. Phrase search for 'information retrieval':");
    let results = index.phrase_search("information retrieval");
    print_results(&index, &results);
    
    println!("\n6. Wildcard search for 'learn*':");
    let results = index.wildcard_search("learn*");
    print_results(&index, &results);
    
    println!("\n=== Term Frequency Analysis ===\n");
    analyze_term(&index, "search", doc2);
    analyze_term(&index, "language", doc3);
    analyze_term(&index, "retrieval", doc1);
}

fn print_results(index: &InvertedIndex, results: &[fulltext_index::search::SearchResult]) {
    if results.is_empty() {
        println!("  No results found.");
    } else {
        for (i, result) in results.iter().enumerate() {
            println!("  {}. [Score: {:.3}] {}", 
                i + 1, 
                result.score, 
                result.title
            );
            println!("     Snippet: {}", result.snippet);
        }
    }
}

fn analyze_term(index: &InvertedIndex, term: &str, doc_id: usize) {
    let tf = index.get_term_frequency(term, doc_id);
    let df = index.get_document_frequency(term);
    
    if let Some(doc) = index.get_document(doc_id) {
        println!("Term '{}' in document '{}':", term, doc.title);
        println!("  Term Frequency (TF): {}", tf);
        println!("  Document Frequency (DF): {}", df);
        println!("  Appears in {}/{} documents", df, index.total_documents());
    }
}
