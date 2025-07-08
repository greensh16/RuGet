#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD-optimized operations for better performance
/// Currently focuses on x86_64 with SSE/AVX support

/// Fast memory copying using SIMD instructions when available
#[cfg(target_arch = "x86_64")]
pub fn fast_memcpy(src: &[u8], dst: &mut [u8]) {
    if src.len() != dst.len() {
        dst.copy_from_slice(src);
        return;
    }

    if is_x86_feature_detected!("avx2") && src.len() >= 32 {
        unsafe { fast_memcpy_avx2(src, dst) }
    } else if is_x86_feature_detected!("sse2") && src.len() >= 16 {
        unsafe { fast_memcpy_sse2(src, dst) }
    } else {
        dst.copy_from_slice(src);
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn fast_memcpy(src: &[u8], dst: &mut [u8]) {
    dst.copy_from_slice(src);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fast_memcpy_avx2(src: &[u8], dst: &mut [u8]) {
    let len = src.len();
    let mut i = 0;

    // Process 32-byte chunks with AVX2
    while i + 32 <= len {
        let chunk = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
        _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, chunk);
        i += 32;
    }

    // Copy remaining bytes
    if i < len {
        std::ptr::copy_nonoverlapping(src.as_ptr().add(i), dst.as_mut_ptr().add(i), len - i);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn fast_memcpy_sse2(src: &[u8], dst: &mut [u8]) {
    let len = src.len();
    let mut i = 0;

    // Process 16-byte chunks with SSE2
    while i + 16 <= len {
        let chunk = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
        _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, chunk);
        i += 16;
    }

    // Copy remaining bytes
    if i < len {
        std::ptr::copy_nonoverlapping(src.as_ptr().add(i), dst.as_mut_ptr().add(i), len - i);
    }
}

/// SIMD-optimized search for patterns in byte arrays
#[cfg(target_arch = "x86_64")]
pub fn find_pattern_simd(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    if is_x86_feature_detected!("avx2") && haystack.len() >= 32 {
        unsafe { find_pattern_avx2(haystack, needle) }
    } else if is_x86_feature_detected!("sse2") && haystack.len() >= 16 {
        unsafe { find_pattern_sse2(haystack, needle) }
    } else {
        find_pattern_scalar(haystack, needle)
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn find_pattern_simd(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    find_pattern_scalar(haystack, needle)
}

fn find_pattern_scalar(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn find_pattern_avx2(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() == 1 {
        let target = _mm256_set1_epi8(needle[0] as i8);
        let mut i = 0;
        
        while i + 32 <= haystack.len() {
            let chunk = _mm256_loadu_si256(haystack.as_ptr().add(i) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(chunk, target);
            let mask = _mm256_movemask_epi8(cmp);
            
            if mask != 0 {
                let offset = mask.trailing_zeros() as usize;
                return Some(i + offset);
            }
            
            i += 32;
        }
    }
    
    // Fallback to scalar for remaining bytes or multi-byte patterns
    let i = 0; // Initialize for fallback
    if let Some(pos) = find_pattern_scalar(&haystack[i..], needle) {
        Some(i + pos)
    } else {
        None
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn find_pattern_sse2(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() == 1 {
        let target = _mm_set1_epi8(needle[0] as i8);
        let mut i = 0;
        
        while i + 16 <= haystack.len() {
            let chunk = _mm_loadu_si128(haystack.as_ptr().add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, target);
            let mask = _mm_movemask_epi8(cmp);
            
            if mask != 0 {
                let offset = mask.trailing_zeros() as usize;
                return Some(i + offset);
            }
            
            i += 16;
        }
    }
    
    // Fallback to scalar for remaining bytes or multi-byte patterns
    let i = 0; // Initialize for fallback
    if let Some(pos) = find_pattern_scalar(&haystack[i..], needle) {
        Some(i + pos)
    } else {
        None
    }
}

/// Fast HTTP header parsing using SIMD
pub fn find_header_end(data: &[u8]) -> Option<usize> {
    // Look for "\r\n\r\n" pattern
    find_pattern_simd(data, b"\r\n\r\n")
}

/// Fast content-length extraction from headers
pub fn extract_content_length(headers: &[u8]) -> Option<usize> {
    // Look for "Content-Length:" (case insensitive)
    let patterns = [
        b"Content-Length:",
        b"content-length:",
        b"CONTENT-LENGTH:",
    ];
    
    for pattern in &patterns {
        if let Some(pos) = find_pattern_simd(headers, *pattern) {
            let after_colon = pos + pattern.len();
            if let Some(line_end) = find_pattern_simd(&headers[after_colon..], b"\r\n") {
                let value_slice = &headers[after_colon..after_colon + line_end];
                let value_str = std::str::from_utf8(value_slice).ok()?.trim();
                return value_str.parse().ok();
            }
        }
    }
    
    None
}

/// Optimized buffer operations
pub struct SIMDBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl SIMDBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Optimized append operation
    pub fn append(&mut self, src: &[u8]) {
        let old_len = self.data.len();
        self.data.resize(old_len + src.len(), 0);
        fast_memcpy(src, &mut self.data[old_len..]);
    }

    /// Get immutable slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Clear buffer for reuse
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for SIMDBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_memcpy() {
        let src = b"Hello, SIMD world!";
        let mut dst = vec![0u8; src.len()];
        
        fast_memcpy(src, &mut dst);
        assert_eq!(src, dst.as_slice());
    }

    #[test]
    fn test_find_pattern_simd() {
        let haystack = b"HTTP/1.1 200 OK\r\nContent-Length: 42\r\n\r\nHello World";
        
        assert_eq!(find_pattern_simd(haystack, b"\r\n\r\n"), Some(35));
        assert_eq!(find_pattern_simd(haystack, b"Content-Length:"), Some(17));
        assert_eq!(find_pattern_simd(haystack, b"Not Found"), None);
    }

    #[test]
    fn test_extract_content_length() {
        let headers = b"HTTP/1.1 200 OK\r\nContent-Length: 42\r\nConnection: close\r\n";
        assert_eq!(extract_content_length(headers), Some(42));

        let headers = b"HTTP/1.1 200 OK\r\ncontent-length: 1024\r\n";
        assert_eq!(extract_content_length(headers), Some(1024));
    }

    #[test]
    fn test_simd_buffer() {
        let mut buffer = SIMDBuffer::new();
        buffer.append(b"Hello");
        buffer.append(b", ");
        buffer.append(b"SIMD!");
        
        assert_eq!(buffer.as_slice(), b"Hello, SIMD!");
        assert_eq!(buffer.len(), 12);
    }
}
