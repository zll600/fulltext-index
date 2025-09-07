# Fulltext Index Demo in Rust

A simple implementation of a fulltext search index in Rust for learning purposes. This project demonstrates the fundamental concepts of information retrieval and search engine technology.

## Features

### Core Components

1. **Inverted Index**: Maps terms to document IDs for efficient search
2. **Document Storage**: Stores and manages documents with metadata
3. **Position Index**: Tracks term positions for phrase search support
4. **Tokenizer**: Processes text with stop word filtering and normalization
5. **TF-IDF Scoring**: Ranks search results by relevance

### Search Capabilities

- **Simple Term Search**: Find documents containing specific terms
- **Boolean Queries**: Support for AND, OR, NOT operators
- **Phrase Search**: Find exact phrases in documents
- **Wildcard Search**: Pattern matching with * operator
- **TF-IDF Ranking**: Results sorted by relevance score

## Project Structure

```
src/
├── lib.rs          # Library entry point
├── main.rs         # Demo application
├── index.rs        # Inverted index implementation
├── document.rs     # Document storage and management
├── tokenizer.rs    # Text processing and tokenization
└── search.rs       # Search functionality and scoring
```

## Key Concepts Demonstrated

### Inverted Index
The core data structure that maps terms to document IDs, enabling fast lookups.

### Text Processing Pipeline
1. Tokenization: Split text into words
2. Normalization: Convert to lowercase
3. Stop word filtering: Remove common words
4. Optional stemming: Reduce words to root forms

### TF-IDF Scoring
- **Term Frequency (TF)**: How often a term appears in a document
- **Inverse Document Frequency (IDF)**: How rare a term is across all documents
- **TF-IDF**: Combined score for relevance ranking

### Position Index
Tracks where terms appear in documents, enabling:
- Phrase search
- Proximity search (future enhancement)
- Snippet generation

## Running the Demo

```bash
cargo run
```

The demo will:
1. Create an in-memory index
2. Add sample documents about information retrieval, search engines, NLP, databases, and machine learning
3. Demonstrate various search types with results
4. Show term frequency analysis

## Example Output

```
=== Search Examples ===

1. Simple term search for 'search':
  1. [Score: 0.518] Search Engine Technology
     Snippet: Modern search engines use inverted indexes...

3. Boolean AND search for 'search' AND 'engine':
  1. [Score: 0.699] Search Engine Technology
     Snippet: Modern search engines use inverted indexes...

5. Phrase search for 'information retrieval':
  1. [Score: 1.000] Introduction to Information Retrieval
     Snippet: Information retrieval is the process...
```

## Learning Resources

This demo covers fundamental concepts from:
- Information Retrieval textbooks
- Search engine architecture
- Natural language processing basics
- Database indexing techniques

## Future Enhancements

Potential improvements to explore:
- Persistence: Save/load index from disk
- Compression: Reduce memory usage
- Advanced stemming: Use linguistic algorithms
- Query expansion: Synonyms and related terms
- Faceted search: Filter by categories
- Real-time indexing: Update without rebuild
- Distributed indexing: Scale across machines

## Dependencies

Currently uses only Rust standard library. Optional dependencies for enhancements:
- `serde`: Serialization for persistence
- `rust-stemmers`: Advanced stemming algorithms
- `unicode-segmentation`: Better tokenization
- `memmap2`: Memory-mapped files for large indexes

## License

This is a learning project for educational purposes.