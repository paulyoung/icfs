// Based on:
// * https://users.rust-lang.org/t/existing-tests-for-read-write-and-seek-traits/72991/2
// * https://github.com/rust-lang/rust/blob/a2ebd5a1f12f4242edf66cbbd471c421bec62753/library/std/src/io/cursor/tests.rs

#![feature(write_all_vectored)]

use ic_cdk_macros::{init, query, update};
use std::io::{IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write};

thread_local! {
    static STABLE_MEMORY: std::cell::RefCell<icfs::StableMemory>
        = std::cell::RefCell::new(icfs::StableMemory::default());
}

fn setup() {
    STABLE_MEMORY.with(|stable_memory| {
        let mut stable_memory = *stable_memory.borrow();
        let capacity = icfs::StableMemory::capacity();
        let b: &[_] = &vec![0; capacity];

        ic_cdk::api::stable::stable64_write(0, &b);
        assert_eq!(&icfs::StableMemory::bytes()[..], b);

        stable_memory.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(stable_memory.stream_position().unwrap(), 0);
    })
}

#[update]
fn test_writer() {
    setup();
    STABLE_MEMORY.with(|stable_memory| {
        let mut stable_memory = *stable_memory.borrow();
        assert_eq!(stable_memory.write(&[0]).unwrap(), 1);
        assert_eq!(stable_memory.write(&[1, 2, 3]).unwrap(), 3);
        assert_eq!(stable_memory.write(&[4, 5, 6, 7]).unwrap(), 4);
        stable_memory
            .write_all_vectored(&mut [
                IoSlice::new(&[]),
                IoSlice::new(&[8, 9]),
                IoSlice::new(&[10]),
            ])
            .unwrap();
        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(&icfs::StableMemory::bytes()[0..11], b);
    })
}

#[update]
fn test_writer_vectored() {
    setup();
    STABLE_MEMORY.with(|stable_memory| {
        let mut stable_memory = *stable_memory.borrow();
        assert_eq!(stable_memory.stream_position().unwrap(), 0);

        stable_memory.write_all_vectored(&mut [IoSlice::new(&[0])]).unwrap();
        assert_eq!(stable_memory.stream_position().unwrap(), 1);

        stable_memory.write_all_vectored(&mut [IoSlice::new(&mut [1, 2, 3]), IoSlice::new(&mut [4, 5, 6, 7]),]).unwrap();
        assert_eq!(stable_memory.stream_position().unwrap(), 8);

        stable_memory.write_all_vectored(&mut []).unwrap();
        assert_eq!(stable_memory.stream_position().unwrap(), 8);

        stable_memory.write_all_vectored(&mut [IoSlice::new(&[8, 9])]).unwrap();
        stable_memory.write_all_vectored(&mut [IoSlice::new(&[10])]).unwrap();

        let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(&icfs::StableMemory::bytes()[0..9], b);
    })
}

#[update]
fn test_writer_seek() {
    setup();
    STABLE_MEMORY.with(|stable_memory| {
        let mut stable_memory = *stable_memory.borrow();

        assert_eq!(stable_memory.stream_position().unwrap(), 0);
        assert_eq!(stable_memory.write(&[1]).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), 1);

        assert_eq!(stable_memory.seek(SeekFrom::Start(2)).unwrap(), 2);
        assert_eq!(stable_memory.stream_position().unwrap(), 2);
        assert_eq!(stable_memory.write(&[2]).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), 3);

        assert_eq!(stable_memory.seek(SeekFrom::Current(-2)).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), 1);
        assert_eq!(stable_memory.write(&[3]).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), 2);

        let capacity = icfs::StableMemory::capacity();

        assert_eq!(stable_memory.seek(SeekFrom::End(-1)).unwrap(), capacity as u64 - 1);
        assert_eq!(stable_memory.stream_position().unwrap(), capacity as u64 - 1);
        assert_eq!(stable_memory.write(&[4]).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), capacity as u64);

        let b: &[_] = &[1, 3, 2, 0, 0, 0, 0, 0];
        assert_eq!(&icfs::StableMemory::bytes()[0..8], b);

        let b: &[_] = &[0, 0, 0, 0, 0, 0, 0, 4];
        assert_eq!(&icfs::StableMemory::bytes()[(capacity - 8)..], b);
    })
}

#[update]
fn test_reader() {
    setup();
    STABLE_MEMORY.with(|stable_memory| {
        let mut stable_memory = *stable_memory.borrow();
        stable_memory.write(&[0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        stable_memory.seek(SeekFrom::Start(0)).unwrap();

        let mut buf = [];
        assert_eq!(stable_memory.read(&mut buf).unwrap(), 0);
        assert_eq!(stable_memory.stream_position().unwrap(), 0);

        let mut buf = [0];
        assert_eq!(stable_memory.read(&mut buf).unwrap(), 1);
        assert_eq!(stable_memory.stream_position().unwrap(), 1);

        let b: &[_] = &[0];
        assert_eq!(buf, b);

        let mut buf = [0; 4];
        assert_eq!(stable_memory.read(&mut buf).unwrap(), 4);
        assert_eq!(stable_memory.stream_position().unwrap(), 5);

        let b: &[_] = &[1, 2, 3, 4];
        assert_eq!(buf, b);
        assert_eq!(stable_memory.read(&mut buf).unwrap(), 4);

        let b: &[_] = &[5, 6, 7, 0];
        assert_eq!(buf, b);

        let b: &[_] = &[5, 6, 7];
        assert_eq!(&buf[..3], b);

        assert_eq!(stable_memory.read(&mut buf).unwrap(), 4);
        let b: &[_] = &[0, 0, 0, 0];
        assert_eq!(buf, b);
    })
}
