use std::path::{Path, PathBuf};

pub fn remove_common_ancestor(repo: &Path, common: Option<&Path>) -> String {
    if let Some(prefix) = common {
        repo.strip_prefix(prefix)
            .unwrap_or(repo)
            .display()
            .to_string()
    } else {
        repo.display().to_string()
    }
}

pub fn common_ancestor(ss: &[PathBuf]) -> PathBuf {
    if ss.is_empty() || ss.len() == 1 {
        return PathBuf::new();
    }
    let mut iterators: Vec<_> = ss.iter().map(|p| p.components()).collect();
    let mut prefix: Vec<std::path::Component> = Vec::new();
    'outer: loop {
        let mut next: Option<std::path::Component> = None;
        for it in iterators.iter_mut() {
            if let Some(c) = it.next() {
                if let Some(prev) = next {
                    if prev != c {
                        break 'outer;
                    }
                } else {
                    next = Some(c);
                }
            } else {
                break 'outer;
            }
        }
        if let Some(c) = next {
            prefix.push(c);
        } else {
            break;
        }
    }
    prefix.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::common_ancestor;

    #[test]
    fn test_common_ancestor() {
        assert_eq!(
            common_ancestor(&[
                std::path::PathBuf::from("/home/cdavison/code"),
                std::path::PathBuf::from("/home/cdavison/code/recipes"),
                std::path::PathBuf::from("/home/cdavison/strathclyde")
            ]),
            std::path::PathBuf::from("/home/cdavison/")
        );
    }
}
