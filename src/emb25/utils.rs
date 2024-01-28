use std::collections::HashMap;
use tantivy::tokenizer::*;

pub fn tokenize(text: &str) -> Vec<String> {
    let mut en_stem = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(40))
        .filter(Stemmer::new(Language::English))
        .build();
    let mut tokens = Vec::new();
    en_stem.token_stream(text).process(&mut |token| {
        tokens.push(token.text.clone());
    });

    tokens
}

pub fn group_by(tokens: &Vec<String>) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for token in tokens {
        let count = map.entry(token.clone()).or_insert(0);
        *count += 1;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "This is a test";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_empty_tokenize() {
        let text = "";
        let tokens = tokenize(text);
        let vector: Vec<String> = vec![];
        assert_eq!(tokens, vector);
    }
}
