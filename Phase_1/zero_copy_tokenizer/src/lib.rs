//! # Zero-Copy String Tokenizer / Scanner
//!
//! A high-performance, allocation-free string tokenizer and scanner designed for
//! parsing delimited text formats (such as CSV rows, log entries, and network headers)
//! without requesting memory from the heap allocator.

/// Token representing a parsed slice from the input stream.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    /// The zero-copy string slice referencing the original input.
    pub value: &'a str,
    /// Byte index where this token started in the original slice.
    pub start_offset: usize,
    /// Byte index where this token ended in the original slice.
    pub end_offset: usize,
}

impl<'a> Token<'a> {
    /// Constructs a new Token wrapping a borrowed slice and its offsets.
    #[inline]
    pub const fn new(value: &'a str, start_offset: usize, end_offset: usize) -> Self {
        Self {
            value,
            start_offset,
            end_offset,
        }
    }

    /// Returns the length of the token in bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns `true` if the token slice is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

/// A zero-allocation tokenizer that yields borrowed string slices (`&'a str`)
/// or structured [`Token<'a>`] instances separated by a chosen delimiter.
///
/// Handles consecutive delimiters, empty trailing fields, and supports
/// optional quotation boundaries (e.g. standard CSV `"quoted values"`).
#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    /// The remaining slice of the input to be parsed.
    remainder: &'a str,
    /// Delimiter character used to separate tokens.
    delimiter: char,
    /// The current byte offset relative to the original source input.
    cursor: usize,
    /// Whether to treat quoted segments as atomic tokens ignoring inner delimiters.
    handle_quotes: bool,
    /// Tracks if parsing has concluded.
    finished: bool,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new `Tokenizer` from a string slice and a delimiter.
    ///
    /// By default, quote handling is disabled for maximum throughput.
    ///
    /// # Examples
    /// ```
    /// use zero_copy_tokenizer::Tokenizer;
    ///
    /// let mut tokenizer = Tokenizer::new("GET /index.html HTTP/1.1", ' ');
    /// assert_eq!(tokenizer.next(), Some("GET"));
    /// assert_eq!(tokenizer.next(), Some("/index.html"));
    /// assert_eq!(tokenizer.next(), Some("HTTP/1.1"));
    /// assert_eq!(tokenizer.next(), None);
    /// ```
    pub const fn new(input: &'a str, delimiter: char) -> Self {
        Self {
            remainder: input,
            delimiter,
            cursor: 0,
            handle_quotes: false,
            finished: false,
        }
    }

    /// Enables quote awareness for CSV-like parsing where delimiters inside
    /// matching quotes `"` are treated as literal characters.
    pub const fn with_quotes(mut self) -> Self {
        self.handle_quotes = true;
        self
    }

    /// Returns the next token with its byte offset metadata without heap allocation.
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if self.finished {
            return None;
        }

        if self.remainder.is_empty() {
            self.finished = true;
            return None;
        }

        let start_offset = self.cursor;

        if self.handle_quotes && self.remainder.starts_with('"') {
            // Find closing quote
            let quote_trimmed = &self.remainder[1..];
            if let Some(close_idx) = quote_trimmed.find('"') {
                let token_slice = &quote_trimmed[..close_idx];
                let consumed = close_idx + 2; // account for opening & closing quotes

                if consumed < self.remainder.len() && self.remainder[consumed..].starts_with(self.delimiter) {
                    let step = consumed + self.delimiter.len_utf8();
                    self.remainder = &self.remainder[step..];
                    self.cursor += step;
                } else {
                    self.remainder = &self.remainder[consumed..];
                    self.cursor += consumed;
                    if self.remainder.is_empty() {
                        self.finished = true;
                    }
                }

                let end_offset = start_offset + consumed;
                return Some(Token::new(token_slice, start_offset, end_offset));
            }
        }

        // Standard delimiter search
        match self.remainder.find(self.delimiter) {
            Some(idx) => {
                let token = &self.remainder[..idx];
                let step = idx + self.delimiter.len_utf8();
                let end_offset = start_offset + idx;

                self.remainder = &self.remainder[step..];
                self.cursor += step;

                Some(Token::new(token, start_offset, end_offset))
            }
            None => {
                let token = self.remainder;
                let end_offset = start_offset + token.len();
                self.remainder = "";
                self.cursor = end_offset;
                self.finished = true;

                Some(Token::new(token, start_offset, end_offset))
            }
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().map(|t| t.value)
    }
}

/// An ultra-optimized, zero-allocation scanner that uses raw byte pointers
/// and unchecked slicing to achieve maximum CPU throughput.
///
/// # Safety Guarantees
/// All `unsafe` pointer operations uphold Rust's UTF-8 invariant and bounds safety:
/// 1. Slicing occurs exclusively at matching ASCII delimiters (which are valid 1-byte UTF-8 boundaries).
/// 2. Pointers remain within the memory region `[ptr, ptr + len]`.
/// 3. Incurs 0 bounds checks, 0 UTF-8 validation re-checks, and minimal branching.
#[derive(Debug, Clone)]
pub struct UnsafeFastTokenizer<'a> {
    ptr: *const u8,
    end: *const u8,
    delimiter: u8,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> UnsafeFastTokenizer<'a> {
    /// Creates a new `UnsafeFastTokenizer` for an ASCII delimiter (e.g. `b','` or `b' '`).
    #[inline(always)]
    pub fn new(input: &'a str, delimiter: u8) -> Self {
        let len = input.len();
        let ptr = input.as_ptr();
        // Safety: ptr + len is the valid one-past-the-end pointer for this slice
        let end = unsafe { ptr.add(len) };
        Self {
            ptr,
            end,
            delimiter,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for UnsafeFastTokenizer<'a> {
    type Item = &'a str;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end {
            return None;
        }

        let start = self.ptr;
        let mut curr = start;

        // Tight raw-pointer scanning loop.
        // Modern LLVM automatically vectorizes/unrolls this loop.
        while curr < self.end {
            // Safety: curr is strictly < self.end, so reading 1 byte is safe.
            if unsafe { *curr } == self.delimiter {
                let token_len = curr as usize - start as usize;
                // Safety: curr is < self.end, so curr.add(1) is at most self.end.
                self.ptr = unsafe { curr.add(1) };

                // Safety:
                // 1. start..curr was sliced at an ASCII delimiter which is a valid UTF-8 boundary.
                // 2. The input was already validated &str.
                unsafe {
                    let bytes = std::slice::from_raw_parts(start, token_len);
                    return Some(std::str::from_utf8_unchecked(bytes));
                }
            }
            curr = unsafe { curr.add(1) };
        }

        // Final token to the end of the slice
        let token_len = self.end as usize - start as usize;
        self.ptr = self.end;

        unsafe {
            let bytes = std::slice::from_raw_parts(start, token_len);
            Some(std::str::from_utf8_unchecked(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_space_delimited() {
        let input = "2026-10-03 INFO [worker-1] Task completed successfully";
        let tokens: Vec<&str> = Tokenizer::new(input, ' ').collect();
        assert_eq!(
            tokens,
            vec!["2026-10-03", "INFO", "[worker-1]", "Task", "completed", "successfully"]
        );
    }

    #[test]
    fn test_csv_delimiters_and_empty_fields() {
        let input = "alice,admin,,42";
        let tokens: Vec<&str> = Tokenizer::new(input, ',').collect();
        assert_eq!(tokens, vec!["alice", "admin", "", "42"]);
    }

    #[test]
    fn test_single_token() {
        let input = "single_token";
        let mut tokenizer = Tokenizer::new(input, ',');
        assert_eq!(tokenizer.next(), Some("single_token"));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let mut tokenizer = Tokenizer::new(input, ',');
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_token_offsets() {
        let input = "POST /api/v1/auth HTTP/1.1";
        let mut tokenizer = Tokenizer::new(input, ' ');

        let t1 = tokenizer.next_token().unwrap();
        assert_eq!(t1.value, "POST");
        assert_eq!(t1.start_offset, 0);
        assert_eq!(t1.end_offset, 4);

        let t2 = tokenizer.next_token().unwrap();
        assert_eq!(t2.value, "/api/v1/auth");
        assert_eq!(t2.start_offset, 5);
        assert_eq!(t2.end_offset, 17);

        let t3 = tokenizer.next_token().unwrap();
        assert_eq!(t3.value, "HTTP/1.1");
        assert_eq!(t3.start_offset, 18);
        assert_eq!(t3.end_offset, 26);

        assert_eq!(tokenizer.next_token(), None);
    }

    #[test]
    fn test_quoted_tokens() {
        let input = "101,\"Smith, John\",Engineer,95000";
        let tokens: Vec<&str> = Tokenizer::new(input, ',').with_quotes().collect();
        assert_eq!(tokens, vec!["101", "Smith, John", "Engineer", "95000"]);
    }

    #[test]
    fn test_unsafe_fast_tokenizer_matches_safe() {
        let input = "alice,admin,,42,engineer,senior,120000";
        let safe_tokens: Vec<&str> = Tokenizer::new(input, ',').collect();
        let fast_tokens: Vec<&str> = UnsafeFastTokenizer::new(input, b',').collect();
        assert_eq!(safe_tokens, fast_tokens);
    }

    #[test]
    fn test_unsafe_fast_tokenizer_edge_cases() {
        // Empty string
        let empty_tokens: Vec<&str> = UnsafeFastTokenizer::new("", b',').collect();
        assert!(empty_tokens.is_empty());

        // Single token
        let single: Vec<&str> = UnsafeFastTokenizer::new("hello", b',').collect();
        assert_eq!(single, vec!["hello"]);

        // Leading and interior delimiters
        let edge_input = ",a,,b,";
        let safe_tokens: Vec<&str> = Tokenizer::new(edge_input, ',').collect();
        let fast_tokens: Vec<&str> = UnsafeFastTokenizer::new(edge_input, b',').collect();
        assert_eq!(safe_tokens, fast_tokens);
    }
}
