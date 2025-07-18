use std::collections::HashMap;
use anyhow::Result;
use regex::Regex;

/// Text search utilities for book content
pub struct TextSearch {
    stop_words: Vec<String>,
    stemmer: Option<Box<dyn Stemmer>>,
}

impl TextSearch {
    /// Create a new text search instance
    pub fn new() -> Self {
        Self {
            stop_words: default_stop_words(),
            stemmer: None,
        }
    }

    /// Search for a query in text content
    pub fn search(&self, content: &str, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        // Normalize query
        let normalized_query = self.normalize_text(query);
        let query_terms: Vec<&str> = normalized_query.split_whitespace().collect();
        
        if query_terms.is_empty() {
            return Ok(results);
        }

        // Search for exact phrase first
        if query_terms.len() > 1 {
            let phrase_results = self.search_phrase(content, query)?;
            results.extend(phrase_results);
        }

        // Search for individual terms
        for term in &query_terms {
            let term_results = self.search_term(content, term)?;
            results.extend(term_results);
        }

        // Remove duplicates and sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results.dedup_by(|a, b| a.start_offset == b.start_offset);

        Ok(results)
    }

    /// Search for an exact phrase in content
    fn search_phrase(&self, content: &str, phrase: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let normalized_content = self.normalize_text(content);
        let normalized_phrase = self.normalize_text(phrase);
        
        let regex = Regex::new(&regex::escape(&normalized_phrase))?;
        
        for mat in regex.find_iter(&normalized_content) {
            let context = self.extract_context(content, mat.start(), mat.end());
            results.push(SearchResult {
                start_offset: mat.start(),
                end_offset: mat.end(),
                matched_text: content[mat.start()..mat.end()].to_string(),
                context,
                relevance_score: 1.0, // Exact phrase match gets highest score
                match_type: MatchType::ExactPhrase,
            });
        }

        Ok(results)
    }

    /// Search for a single term in content
    fn search_term(&self, content: &str, term: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let normalized_content = self.normalize_text(content);
        let normalized_term = self.normalize_text(term);
        
        // Skip stop words
        if self.stop_words.contains(&normalized_term) {
            return Ok(results);
        }

        // Word boundary search
        let pattern = format!(r"\b{}\b", regex::escape(&normalized_term));
        let regex = Regex::new(&pattern)?;
        
        for mat in regex.find_iter(&normalized_content) {
            let context = self.extract_context(content, mat.start(), mat.end());
            results.push(SearchResult {
                start_offset: mat.start(),
                end_offset: mat.end(),
                matched_text: content[mat.start()..mat.end()].to_string(),
                context,
                relevance_score: 0.7, // Single term match
                match_type: MatchType::Term,
            });
        }

        Ok(results)
    }

    /// Extract context around a match
    fn extract_context(&self, content: &str, start: usize, end: usize) -> String {
        let context_size = 100; // Characters before and after
        
        let context_start = if start > context_size { start - context_size } else { 0 };
        let context_end = if end + context_size < content.len() { 
            end + context_size 
        } else { 
            content.len() 
        };
        
        let mut context = content[context_start..context_end].to_string();
        
        // Add ellipsis if truncated
        if context_start > 0 {
            context = format!("...{}", context);
        }
        if context_end < content.len() {
            context = format!("{}...", context);
        }
        
        context
    }

    /// Normalize text for searching
    fn normalize_text(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    /// Build search index from text content
    pub fn build_index(&self, content: &str) -> SearchIndex {
        let mut word_positions: HashMap<String, Vec<usize>> = HashMap::new();
        let normalized_content = self.normalize_text(content);
        
        let mut current_pos = 0;
        for word in normalized_content.split_whitespace() {
            if !self.stop_words.contains(&word.to_string()) {
                let positions = word_positions.entry(word.to_string()).or_insert_with(Vec::new);
                positions.push(current_pos);
            }
            current_pos += word.len() + 1; // +1 for space
        }
        
        SearchIndex {
            word_positions,
            content: content.to_string(),
        }
    }

    /// Search using pre-built index
    pub fn search_with_index(&self, index: &SearchIndex, query: &str) -> Result<Vec<SearchResult>> {
        let normalized_query = self.normalize_text(query);
        let query_terms: Vec<&str> = normalized_query.split_whitespace().collect();
        
        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        
        // Find positions for each term
        for term in query_terms {
            if let Some(positions) = index.word_positions.get(term) {
                for &pos in positions {
                    let word_end = pos + term.len();
                    let context = self.extract_context(&index.content, pos, word_end);
                    
                    results.push(SearchResult {
                        start_offset: pos,
                        end_offset: word_end,
                        matched_text: term.to_string(),
                        context,
                        relevance_score: 0.8,
                        match_type: MatchType::IndexedTerm,
                    });
                }
            }
        }

        // Sort by relevance and position
        results.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.start_offset.cmp(&b.start_offset))
        });

        Ok(results)
    }

    /// Highlight search terms in text
    pub fn highlight_matches(&self, content: &str, query: &str) -> Result<String> {
        let results = self.search(content, query)?;
        
        if results.is_empty() {
            return Ok(content.to_string());
        }

        let mut highlighted = String::new();
        let mut last_end = 0;
        
        for result in &results {
            // Add text before match
            highlighted.push_str(&content[last_end..result.start_offset]);
            
            // Add highlighted match
            highlighted.push_str(&format!(
                "<mark>{}</mark>", 
                &content[result.start_offset..result.end_offset]
            ));
            
            last_end = result.end_offset;
        }
        
        // Add remaining text
        highlighted.push_str(&content[last_end..]);
        
        Ok(highlighted)
    }

    /// Get search suggestions based on partial query
    pub fn get_suggestions(&self, index: &SearchIndex, partial_query: &str) -> Vec<String> {
        let normalized_partial = self.normalize_text(partial_query);
        let mut suggestions = Vec::new();
        
        for word in index.word_positions.keys() {
            if word.starts_with(&normalized_partial) && word != &normalized_partial {
                suggestions.push(word.clone());
            }
        }
        
        suggestions.sort();
        suggestions.truncate(10); // Limit suggestions
        suggestions
    }
}

/// Search result structure
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub start_offset: usize,
    pub end_offset: usize,
    pub matched_text: String,
    pub context: String,
    pub relevance_score: f64,
    pub match_type: MatchType,
}

/// Types of search matches
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    ExactPhrase,
    Term,
    IndexedTerm,
    Fuzzy,
}

/// Pre-built search index for faster searching
#[derive(Debug, Clone)]
pub struct SearchIndex {
    pub word_positions: HashMap<String, Vec<usize>>,
    pub content: String,
}

/// Trait for text stemming
pub trait Stemmer {
    fn stem(&self, word: &str) -> String;
}

/// Simple stemmer implementation
pub struct SimpleStemmer;

impl Stemmer for SimpleStemmer {
    fn stem(&self, word: &str) -> String {
        let word = word.to_lowercase();
        
        // Very basic stemming rules
        if word.ends_with("ing") && word.len() > 3 {
            return word[..word.len() - 3].to_string();
        }
        if word.ends_with("ed") && word.len() > 2 {
            return word[..word.len() - 2].to_string();
        }
        if word.ends_with("s") && word.len() > 1 {
            return word[..word.len() - 1].to_string();
        }
        
        word
    }
}

/// Default stop words for English
fn default_stop_words() -> Vec<String> {
    vec![
        "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
        "has", "he", "in", "is", "it", "its", "of", "on", "that", "the",
        "to", "was", "will", "with", "but", "or", "not", "have", "had",
        "do", "does", "did", "can", "could", "should", "would", "may",
        "might", "must", "shall", "will", "am", "is", "are", "was", "were",
        "been", "being", "have", "has", "had", "do", "does", "did", "go",
        "goes", "went", "gone", "going", "get", "gets", "got", "gotten",
        "getting", "make", "makes", "made", "making", "take", "takes", "took",
        "taken", "taking", "come", "comes", "came", "coming", "see", "sees",
        "saw", "seen", "seeing", "know", "knows", "knew", "known", "knowing",
        "think", "thinks", "thought", "thinking", "say", "says", "said",
        "saying", "tell", "tells", "told", "telling", "ask", "asks", "asked",
        "asking", "give", "gives", "gave", "given", "giving", "find", "finds",
        "found", "finding", "call", "calls", "called", "calling", "try", "tries",
        "tried", "trying", "need", "needs", "needed", "needing", "feel", "feels",
        "felt", "feeling", "become", "becomes", "became", "becoming", "leave",
        "leaves", "left", "leaving", "put", "puts", "putting", "mean", "means",
        "meant", "meaning", "keep", "keeps", "kept", "keeping", "let", "lets",
        "letting", "begin", "begins", "began", "begun", "beginning", "seem",
        "seems", "seemed", "seeming", "turn", "turns", "turned", "turning",
        "start", "starts", "started", "starting", "show", "shows", "showed",
        "shown", "showing", "hear", "hears", "heard", "hearing", "play", "plays",
        "played", "playing", "run", "runs", "ran", "running", "move", "moves",
        "moved", "moving", "live", "lives", "lived", "living", "believe",
        "believes", "believed", "believing", "hold", "holds", "held", "holding",
        "bring", "brings", "brought", "bringing", "happen", "happens", "happened",
        "happening", "write", "writes", "wrote", "written", "writing", "provide",
        "provides", "provided", "providing", "sit", "sits", "sat", "sitting",
        "stand", "stands", "stood", "standing", "lose", "loses", "lost", "losing",
        "pay", "pays", "paid", "paying", "meet", "meets", "met", "meeting",
        "include", "includes", "included", "including", "continue", "continues",
        "continued", "continuing", "set", "sets", "setting", "learn", "learns",
        "learned", "learning", "change", "changes", "changed", "changing",
        "lead", "leads", "led", "leading", "understand", "understands", "understood",
        "understanding", "watch", "watches", "watched", "watching", "follow",
        "follows", "followed", "following", "stop", "stops", "stopped", "stopping",
        "create", "creates", "created", "creating", "speak", "speaks", "spoke",
        "spoken", "speaking", "read", "reads", "reading", "allow", "allows",
        "allowed", "allowing", "add", "adds", "added", "adding", "spend", "spends",
        "spent", "spending", "grow", "grows", "grew", "grown", "growing", "open",
        "opens", "opened", "opening", "walk", "walks", "walked", "walking",
        "win", "wins", "won", "winning", "offer", "offers", "offered", "offering",
        "remember", "remembers", "remembered", "remembering", "love", "loves",
        "loved", "loving", "consider", "considers", "considered", "considering",
        "appear", "appears", "appeared", "appearing", "buy", "buys", "bought",
        "buying", "wait", "waits", "waited", "waiting", "serve", "serves",
        "served", "serving", "die", "dies", "died", "dying", "send", "sends",
        "sent", "sending", "expect", "expects", "expected", "expecting", "build",
        "builds", "built", "building", "stay", "stays", "stayed", "staying",
        "fall", "falls", "fell", "fallen", "falling", "cut", "cuts", "cutting",
        "reach", "reaches", "reached", "reaching", "kill", "kills", "killed",
        "killing", "remain", "remains", "remained", "remaining"
    ].into_iter().map(|s| s.to_string()).collect()
}

impl Default for TextSearch {
    fn default() -> Self {
        Self::new()
    }
}