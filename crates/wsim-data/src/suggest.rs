//! "Did you mean …?" suggestions for misspelled keys.

/// Returns the closest candidate if it is close enough to be a likely typo.
pub fn closest<'a>(input: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (input.chars().count() / 3).max(1);
    candidates
        .into_iter()
        .map(|c| (distance(&input.to_lowercase(), &c.to_lowercase()), c))
        .filter(|&(d, _)| d <= limit)
        .min_by_key(|&(d, c)| (d, c))
        .map(|(_, c)| c)
}

/// Levenshtein distance in characters.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1; b.len() + 1];
        for (j, &cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        previous = current;
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_typos() {
        let candidates = ["roheisen", "stahl", "draht"];
        assert_eq!(closest("roheisn", candidates), Some("roheisen"));
        assert_eq!(closest("Stahl", candidates), Some("stahl"));
        assert_eq!(closest("kupfer", candidates), None);
    }

    #[test]
    fn measures_distance() {
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("äöü", "aöü"), 1);
    }
}
