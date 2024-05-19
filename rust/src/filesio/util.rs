use std::cmp::min;

// Implements reading from one slice to another, maintaining the start position
// in `pos`.
pub fn read_from_slice(pos: &mut usize, src: &[u8], dst: &mut [u8]) -> std::io::Result<usize> {
    let bytes_remaining = src.len() - *pos;
    let n = min(dst.len(), bytes_remaining);
    dst[..n].copy_from_slice(&src[*pos..*pos + n]);
    *pos += n;
    Ok(n)
}
