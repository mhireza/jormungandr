use assert_fs::assert::IntoPathPredicate;
use predicates::prelude::*;
use std::path::Path;

pub fn file_exists_and_not_empty() -> impl Predicate<Path> {
    predicate::path::exists().and([].into_path().not())
}
