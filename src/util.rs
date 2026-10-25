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

/// Longest path containing every repo. A single repo uses its parent, so its
/// own name stays visible.
pub fn common_ancestor(paths: &[PathBuf]) -> PathBuf {
    let Some(first) = paths.first() else {
        return PathBuf::new();
    };
    let mut prefix = if paths.len() == 1 {
        first.parent().unwrap_or(first)
    } else {
        first.as_path()
    };
    while !paths.iter().all(|p| p.starts_with(prefix)) {
        match prefix.parent() {
            Some(parent) => prefix = parent,
            None => return PathBuf::new(),
        }
    }
    prefix.to_path_buf()
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
        assert_eq!(
            common_ancestor(&[std::path::PathBuf::from("/home/cdavison/code")]),
            std::path::PathBuf::from("/home/cdavison")
        );
    }
}
