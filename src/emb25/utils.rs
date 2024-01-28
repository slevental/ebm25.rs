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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "This is a test";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }
}