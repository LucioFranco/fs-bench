#![feature(test)]

extern crate test;

use test::Bencher;

const TOTAL: u64 = 256;

#[bench]
fn buffered(b: &mut Bencher) {
    let data = Data([12; SIZE]);

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("bench_buffered")
        .expect("open file");

    b.bytes = TOTAL * SIZE as u64;
    b.iter(|| write(&file, &data.0));

    std::fs::remove_file("bench_buffered").unwrap();
}

#[bench]
fn direct(b: &mut Bencher) {
    let data = Data([12; SIZE]);

    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT)
        .open("bench_direct")
        .expect("open file");

    b.bytes = TOTAL * SIZE as u64;
    b.iter(|| write(&file, &data.0));

    std::fs::remove_file("bench_direct").unwrap();
}

use io_uring::IoUring;
use std::os::unix::io::RawFd;

#[bench]
fn io_uring(b: &mut Bencher) {
    let data = Data([12; SIZE]);

    let mut ring = IoUring::new(256).unwrap();

    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT)
        .open("bench_io_uring")
        .expect("open file");

    let fd = file.as_raw_fd();

    b.bytes = TOTAL * SIZE as u64;
    b.iter(|| io_uring_write(&mut ring, fd, &data.0));
    // io_uring_write(&mut ring, fd, &data.0);

    let meta = std::fs::metadata("bench_io_uring").unwrap();
    assert_eq!(meta.len(), b.bytes);
    std::fs::remove_file("bench_io_uring").unwrap();
}

const SIZE: usize = 4096 * 128;

#[repr(align(4096))]
struct Data([u8; SIZE]);

fn write(file: &std::fs::File, data: &[u8]) {
    use std::os::unix::fs::FileExt;

    for i in 0..TOTAL {
        file.write_at(&data[..SIZE], i * SIZE as u64).unwrap();
    }
}

fn io_uring_write(ring: &mut IoUring, fd: RawFd, data: &[u8]) {
    use std::io::IoSlice;

    let buf = [IoSlice::new(data)];

    {
        let mut queue = ring.submission().available();

        for i in 0..TOTAL as i64 {
            use io_uring::opcode::Writev;
            let write = Writev::new(
                io_uring::opcode::types::Target::Fd(fd),
                buf.as_ptr() as *const _,
                1,
            )
            .offset(i * SIZE as i64)
            .build();
            if let Err(_) = unsafe { queue.push(write) } {
                panic!("unable to submit");
            }
        }
    }

    ring.submit_and_wait(TOTAL as usize).unwrap();

    // let cqes = ring.completion().available().collect::<Vec<_>>();
    for op in ring.completion().available() {
        assert_eq!(op.result(), 12);
    }

    // assert_eq!(cqes.len(), 256);
}
