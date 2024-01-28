use tantivy::tokenizer::*;


pub fn tokenize(text: &str) -> Vec<String> {
    let en_stem = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .filter(Stemmer::new(Language::English))
        .build();

    let mut tokens = Vec::new();


    for word in text.split_whitespace() {
        tokens.push(word.to_string());
    }
    tokens
}