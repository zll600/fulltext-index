# Fulltext Index Demo Implementation Plan

## Overview
This project implements a simple fulltext index in Rust for learning purposes. It covers the fundamental concepts of information retrieval and search engine technology.

## Phase 1: Core Data Structures
1. **Create inverted index structure** - HashMap<String, Vec<DocumentId>> to map terms to document IDs
2. **Implement document storage** - Store original documents with IDs for retrieval
3. **Create position index** - Track term positions within documents for phrase search

## Phase 2: Text Processing
1. **Implement tokenizer** - Split text into words, handle punctuation
2. **Add case normalization** - Convert to lowercase for case-insensitive search
3. **Create simple stemmer** - Basic suffix removal (optional: use rust-stemmers crate)
4. **Build stop words filter** - Remove common words like "the", "a", "is"

## Phase 3: Indexing Operations
1. **Add document indexing** - Process documents and update inverted index
2. **Implement term frequency (TF)** - Count occurrences of terms in documents
3. **Add document frequency (DF)** - Track how many documents contain each term
4. **Calculate TF-IDF scoring** - Implement relevance ranking algorithm

## Phase 4: Search Functionality
1. **Basic term search** - Find documents containing a single term
2. **Boolean queries** - Support AND, OR, NOT operators
3. **Phrase search** - Find exact phrases using position index
4. **Wildcard search** - Support prefix/suffix matching with * operator

## Phase 5: Persistence & Optimization
1. **Serialize index to disk** - Save/load index using serde
2. **Implement memory-mapped files** - Use memmap2 for large indexes
3. **Add compression** - Compress posting lists to save space
4. **Create simple CLI** - Interactive search interface

## Key Learning Concepts
- **Inverted Index**: Core data structure for fulltext search
- **Text Analysis Pipeline**: Tokenization → Normalization → Stemming
- **Relevance Scoring**: TF-IDF algorithm for ranking results
- **Query Processing**: Boolean logic and phrase matching
- **Performance Trade-offs**: Memory vs disk, speed vs accuracy

## Suggested Dependencies
- `serde` & `serde_json` - Serialization
- `regex` - Pattern matching
- `unicode-segmentation` - Better tokenization
- `rust-stemmers` (optional) - Stemming algorithms
- `memmap2` (optional) - Memory-mapped files
- `clap` - CLI argument parsing

## Implementation Notes
- Start with Phase 1-3 for a basic working index
- Phase 4 adds advanced search capabilities
- Phase 5 is for production-ready features
- Each phase builds on the previous one
- Test with sample documents at each phase