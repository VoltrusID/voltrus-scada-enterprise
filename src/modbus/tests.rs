use crate::models::*;

use super::types::{Backoff, combine_u16, parse_register};

#[test]
fn test_parse_uint16() {
    assert_eq!(
        parse_register(&[1000u16], &RegisterType::Uint16, &Endianness::Big),
        1000.0
    );
}

#[test]
fn test_parse_int16_negative() {
    assert_eq!(
        parse_register(&[0xFFFFu16], &RegisterType::Int16, &Endianness::Big),
        -1.0
    );
}

#[test]
fn test_parse_float32_big_endian() {
    let val = parse_register(&[0x435C, 0x8000], &RegisterType::Float32, &Endianness::Big);
    assert!((val - 220.5).abs() < 0.01);
}

#[test]
fn test_parse_float32_little_endian() {
    let val = parse_register(
        &[0x8000, 0x435C],
        &RegisterType::Float32,
        &Endianness::Little,
    );
    assert!((val - 220.5).abs() < 0.01);
}

#[test]
fn test_combine_big_endian() {
    assert_eq!(combine_u16(0x1234, 0x5678, &Endianness::Big), 0x12345678);
}

#[test]
fn test_combine_little_endian() {
    assert_eq!(combine_u16(0x5678, 0x1234, &Endianness::Little), 0x12345678);
}

#[test]
fn test_backoff_sequence() {
    let mut b = Backoff::new();
    assert_eq!(b.next(), 1000);
    assert_eq!(b.next(), 2000);
    assert_eq!(b.next(), 4000);
}

#[test]
fn test_backoff_resets() {
    let mut b = Backoff::new();
    b.next();
    b.next();
    b.reset();
    assert_eq!(b.next(), 1000);
}

#[test]
fn test_backoff_max() {
    let mut b = Backoff::new();
    for _ in 0..20 {
        b.next();
    }
    assert_eq!(b.next(), 30_000);
}
