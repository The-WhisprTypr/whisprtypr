pub fn to_camel_case(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in words.iter().skip(1) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_uppercase().next().unwrap_or(first));
            result.extend(chars.map(|c| c.to_lowercase().next().unwrap_or(c)));
        }
    }
    result
}

pub fn to_pascal_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut s = first.to_uppercase().to_string();
                    s.extend(chars.map(|c| c.to_lowercase().next().unwrap_or(c)));
                    s
                }
                None => String::new(),
            }
        })
        .collect()
}

pub fn to_snake_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn to_kebab_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn to_constant_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}
