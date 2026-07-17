// Fixture for the Tier-1 sweep: `HashMap` is imported but never used, so rustc
// emits `unused_imports` (StructureOS SOS027). `add` keeps the crate non-empty
// and buildable so the post-fix build gate is meaningful.
use std::collections::HashMap;

/// Sum two numbers. (No use of HashMap — the import above is dead.)
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 3), 5);
    }
}
