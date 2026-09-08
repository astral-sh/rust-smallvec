// Adopted from `tests/test_buf_mut.rs` in the `bytes` crate.

use bytes::BufMut as _;

type SmallVec = smallvec::SmallVec<u8, 8>;

#[test]
fn smallvec_as_mut_buf() {
    let mut buf = SmallVec::with_capacity(64);

    assert_eq!(buf.remaining_mut(), isize::MAX as usize);

    assert!(buf.chunk_mut().len() >= 64);

    buf.put(&b"zomg"[..]);

    assert_eq!(&buf, b"zomg");

    assert_eq!(buf.remaining_mut(), isize::MAX as usize - 4);
    assert_eq!(buf.capacity(), 64);

    for _ in 0..16 {
        buf.put(&b"zomg"[..]);
    }

    assert_eq!(buf.len(), 68);
}

#[test]
fn smallvec_put_bytes() {
    let mut buf = SmallVec::new();
    buf.push(17);
    buf.put_bytes(19, 2);
    assert_eq!([17, 19, 19], &buf[..]);
}

#[test]
fn put_bytes_preserves_prefix_across_capacity_boundaries() {
    for prefix in [0, 3, 8] {
        for count in [0, 1, 5, 8, 9, 64] {
            for value in [0, 19, 255] {
                let mut buf: SmallVec = smallvec::from_elem(17, prefix);
                buf.put_bytes(value, count);
                let mut expected = vec![17; prefix];
                expected.resize(prefix + count, value);
                assert_eq!(&buf[..], expected);
            }
        }
    }
}

#[test]
fn put_bytes_overflow_preserves_contents() {
    let mut buf = SmallVec::from([17, 18, 19]);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        buf.put_bytes(0, usize::MAX);
    }));
    assert!(result.is_err());
    assert_eq!(&buf[..], &[17, 18, 19]);
}

#[test]
fn put_u8() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_u8(33);
    assert_eq!(b"\x21", &buf[..]);
}

#[test]
fn put_u16() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_u16(8532);
    assert_eq!(b"\x21\x54", &buf[..]);

    buf.clear();
    buf.put_u16_le(8532);
    assert_eq!(b"\x54\x21", &buf[..]);
}

#[test]
fn put_int() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_int(0x1020304050607080, 3);
    assert_eq!(b"\x60\x70\x80", &buf[..]);
}

#[test]
#[should_panic(expected = "size too large")]
fn put_int_nbytes_overflow() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_int(0x1020304050607080, 9);
}

#[test]
fn put_int_le() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_int_le(0x1020304050607080, 3);
    assert_eq!(b"\x80\x70\x60", &buf[..]);
}

#[test]
#[should_panic(expected = "size too large")]
fn put_int_le_nbytes_overflow() {
    let mut buf = SmallVec::with_capacity(8);
    buf.put_int_le(0x1020304050607080, 9);
}

#[test]
#[should_panic(expected = "advance out of bounds")]
fn smallvec_advance_mut() {
    let mut buf = SmallVec::with_capacity(8);
    unsafe {
        buf.advance_mut(12);
    }
}
