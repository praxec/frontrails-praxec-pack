//! Fixture god-file: many small, independent, trivially-correct functions so
//! StructureOS's `god_file_score` heuristic (LOC + function-count driven)
//! fires SOS001, and `propose_decomposition` finds clean sibling groups
//! (math, float, string, vec, option). Intentionally oversized — this crate
//! exists only to be scanned and decomposed by a later campaign task.

// ---------------------------------------------------------------------------
// Integer math helpers
// ---------------------------------------------------------------------------

/// Add two integers.
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Subtract `b` from `a`.
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

/// Multiply two integers.
pub fn mul(a: i64, b: i64) -> i64 {
    a * b
}

/// Divide `a` by `b`, returning `None` on division by zero.
pub fn div(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// Remainder of `a` divided by `b`, returning `None` on division by zero.
pub fn rem(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        Some(a % b)
    }
}

/// Smaller of two integers.
pub fn min(a: i64, b: i64) -> i64 {
    if a < b {
        a
    } else {
        b
    }
}

/// Larger of two integers.
pub fn max(a: i64, b: i64) -> i64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Clamp `value` into the inclusive range `[lo, hi]`.
pub fn clamp(value: i64, lo: i64, hi: i64) -> i64 {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

/// Greatest common divisor via the Euclidean algorithm.
pub fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Least common multiple of two integers.
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b) * b).abs()
    }
}

/// Absolute value of an integer.
pub fn abs(a: i64) -> i64 {
    a.abs()
}

/// Sign of an integer: -1, 0, or 1.
pub fn signum(a: i64) -> i64 {
    a.signum()
}

/// Whether an integer is even.
pub fn is_even(a: i64) -> bool {
    a % 2 == 0
}

/// Whether an integer is odd.
pub fn is_odd(a: i64) -> bool {
    a % 2 != 0
}

/// Integer power via repeated squaring semantics of `i64::pow`.
pub fn pow(base: i64, exp: u32) -> i64 {
    base.pow(exp)
}

/// Factorial of a small non-negative integer (iterative, no recursion).
pub fn factorial(n: u64) -> u64 {
    let mut acc: u64 = 1;
    for i in 1..=n {
        acc = acc.saturating_mul(i);
    }
    acc
}

/// `n`-th Fibonacci number (iterative).
pub fn fib(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = a.saturating_add(b);
        a = b;
        b = next;
    }
    a
}

/// Square of an integer.
pub fn square(a: i64) -> i64 {
    a * a
}

/// Cube of an integer.
pub fn cube(a: i64) -> i64 {
    a * a * a
}

/// Integer average of two numbers, rounded toward zero.
pub fn average_two(a: i64, b: i64) -> i64 {
    (a + b) / 2
}

/// Sum of all integers in the inclusive range `[lo, hi]`.
pub fn sum_range(lo: i64, hi: i64) -> i64 {
    if lo > hi {
        return 0;
    }
    let mut total = 0i64;
    let mut cur = lo;
    while cur <= hi {
        total += cur;
        cur += 1;
    }
    total
}

/// Whether an integer is a positive prime (trial division; fine for small n).
pub fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2i64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }
    true
}

/// Count of set bits in a non-negative integer.
pub fn popcount(n: u64) -> u32 {
    n.count_ones()
}

// ---------------------------------------------------------------------------
// Floating-point helpers
// ---------------------------------------------------------------------------

/// Add two floats.
pub fn f_add(a: f64, b: f64) -> f64 {
    a + b
}

/// Subtract `b` from `a`.
pub fn f_sub(a: f64, b: f64) -> f64 {
    a - b
}

/// Multiply two floats.
pub fn f_mul(a: f64, b: f64) -> f64 {
    a * b
}

/// Divide `a` by `b`, returning `None` when `b` is zero.
pub fn f_div(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

/// Round a float to the nearest integer value.
pub fn f_round(a: f64) -> f64 {
    a.round()
}

/// Floor of a float.
pub fn f_floor(a: f64) -> f64 {
    a.floor()
}

/// Ceiling of a float.
pub fn f_ceil(a: f64) -> f64 {
    a.ceil()
}

/// Absolute value of a float.
pub fn f_abs(a: f64) -> f64 {
    a.abs()
}

/// Linear interpolation between `a` and `b` at parameter `t` in `[0, 1]`.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Clamp a float into the inclusive range `[lo, hi]`.
pub fn clampf(value: f64, lo: f64, hi: f64) -> f64 {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

/// Whether two floats are approximately equal within `epsilon`.
pub fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() <= epsilon
}

/// Percentage of `part` relative to `whole`, returning `None` if `whole` is zero.
pub fn percent_of(part: f64, whole: f64) -> Option<f64> {
    if whole == 0.0 {
        None
    } else {
        Some(part / whole * 100.0)
    }
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

/// Reverse a string by Unicode scalar value.
pub fn str_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Uppercase a string.
pub fn str_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Lowercase a string.
pub fn str_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Trim leading/trailing whitespace from a string.
pub fn str_trim(s: &str) -> &str {
    s.trim()
}

/// Length of a string in bytes.
pub fn str_len(s: &str) -> usize {
    s.len()
}

/// Whether a string is empty.
pub fn str_is_empty(s: &str) -> bool {
    s.is_empty()
}

/// Concatenate two strings.
pub fn str_concat(a: &str, b: &str) -> String {
    let mut out = String::with_capacity(a.len() + b.len());
    out.push_str(a);
    out.push_str(b);
    out
}

/// Repeat a string `n` times.
pub fn str_repeat(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Whether a string contains a substring.
pub fn str_contains(s: &str, needle: &str) -> bool {
    s.contains(needle)
}

/// Whether a string starts with a prefix.
pub fn str_starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Whether a string ends with a suffix.
pub fn str_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Replace only the first occurrence of `from` with `to`.
pub fn str_replace_first(s: &str, from: &str, to: &str) -> String {
    s.replacen(from, to, 1)
}

/// Count the whitespace-separated words in a string.
pub fn str_word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

/// Capitalize the first character of a string, leaving the rest untouched.
pub fn str_capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Strip a known prefix from a string, if present.
pub fn str_strip_prefix(s: &str, prefix: &str) -> Option<String> {
    s.strip_prefix(prefix).map(|rest| rest.to_string())
}

/// Join a slice of string slices with a separator.
pub fn str_join(parts: &[&str], sep: &str) -> String {
    parts.join(sep)
}

/// Split a string on a separator and return the pieces as owned strings.
pub fn str_split_owned(s: &str, sep: &str) -> Vec<String> {
    s.split(sep).map(|p| p.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Vec helpers
// ---------------------------------------------------------------------------

/// Sum all elements of an integer slice.
pub fn vec_sum(items: &[i64]) -> i64 {
    items.iter().sum()
}

/// Maximum element of an integer slice, if non-empty.
pub fn vec_max(items: &[i64]) -> Option<i64> {
    items.iter().copied().max()
}

/// Minimum element of an integer slice, if non-empty.
pub fn vec_min(items: &[i64]) -> Option<i64> {
    items.iter().copied().min()
}

/// Average of an integer slice as a float, if non-empty.
pub fn vec_average(items: &[i64]) -> Option<f64> {
    if items.is_empty() {
        None
    } else {
        Some(items.iter().sum::<i64>() as f64 / items.len() as f64)
    }
}

/// Reverse a vec of integers, returning a new vec.
pub fn vec_reverse(items: &[i64]) -> Vec<i64> {
    let mut out = items.to_vec();
    out.reverse();
    out
}

/// Remove consecutive duplicate values from a vec of integers.
pub fn vec_dedup(items: &[i64]) -> Vec<i64> {
    let mut out = items.to_vec();
    out.dedup();
    out
}

/// First element of an integer slice, if any.
pub fn vec_first(items: &[i64]) -> Option<i64> {
    items.first().copied()
}

/// Last element of an integer slice, if any.
pub fn vec_last(items: &[i64]) -> Option<i64> {
    items.last().copied()
}

/// Whether a slice contains a target value.
pub fn vec_contains(items: &[i64], target: i64) -> bool {
    items.contains(&target)
}

/// Append a value to a vec, returning a new vec (non-mutating).
pub fn vec_push_new(items: &[i64], value: i64) -> Vec<i64> {
    let mut out = items.to_vec();
    out.push(value);
    out
}

/// Filter a slice down to its even elements.
pub fn vec_filter_even(items: &[i64]) -> Vec<i64> {
    items.iter().copied().filter(|n| n % 2 == 0).collect()
}

/// Double every element in a slice.
pub fn vec_map_double(items: &[i64]) -> Vec<i64> {
    items.iter().map(|n| n * 2).collect()
}

/// Number of elements in a slice.
pub fn vec_len(items: &[i64]) -> usize {
    items.len()
}

/// Whether a slice is empty.
pub fn vec_is_empty(items: &[i64]) -> bool {
    items.is_empty()
}

/// Concatenate two integer slices into a new vec.
pub fn vec_concat(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out
}

/// Sort a slice of integers ascending, returning a new vec.
pub fn vec_sorted(items: &[i64]) -> Vec<i64> {
    let mut out = items.to_vec();
    out.sort_unstable();
    out
}

/// Deduplicate a slice of integers regardless of order, preserving first-seen order.
pub fn vec_unique(items: &[i64]) -> Vec<i64> {
    let mut seen = Vec::new();
    for &item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}

// ---------------------------------------------------------------------------
// Option helpers
// ---------------------------------------------------------------------------

/// Unwrap an option, or return a fallback default.
pub fn opt_unwrap_or(value: Option<i64>, default: i64) -> i64 {
    value.unwrap_or(default)
}

/// Map `Some(n)` to `Some(n + 1)`, leaving `None` untouched.
pub fn opt_map_add_one(value: Option<i64>) -> Option<i64> {
    value.map(|n| n + 1)
}

/// Whether an option holds a value.
pub fn opt_is_some(value: &Option<i64>) -> bool {
    value.is_some()
}

/// Whether an option is empty.
pub fn opt_is_none(value: &Option<i64>) -> bool {
    value.is_none()
}

/// Chain an option through a doubling step via `and_then`.
pub fn opt_and_then_double(value: Option<i64>) -> Option<i64> {
    value.and_then(|n| n.checked_mul(2))
}

/// Return the option's value, or a type-level default.
pub fn opt_or_default(value: Option<i64>) -> i64 {
    value.unwrap_or_default()
}

/// Keep the value only if it is positive.
pub fn opt_filter_positive(value: Option<i64>) -> Option<i64> {
    value.filter(|n| *n > 0)
}

/// Sum two options together, if both are present.
pub fn opt_zip_sum(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    a.zip(b).map(|(x, y)| x + y)
}

/// Take a value out of an option-like slot, or fall back to a default.
pub fn opt_take_or(mut value: Option<i64>, default: i64) -> i64 {
    value.take().unwrap_or(default)
}

/// Flatten a nested option into a single-level option.
pub fn opt_flatten_two(value: Option<Option<i64>>) -> Option<i64> {
    value.flatten()
}

/// Replace an option's contents, returning the previous value.
pub fn opt_replace(value: &mut Option<i64>, new_value: i64) -> Option<i64> {
    value.replace(new_value)
}

/// Get an option's value or compute one lazily via a closure.
pub fn opt_or_else_zero(value: Option<i64>) -> i64 {
    value.unwrap_or_else(|| 0)
}
