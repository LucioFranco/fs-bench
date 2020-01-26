// use std::fs::File;
// use std::io;
// use std::path::Path;

// const SIZE: usize = 4096 * 128;

// #[repr(align(4096))]
// struct Data {
//     inner: [u8; SIZE],
// }

// pub struct Append<I> {
//     io: I,
//     data: Data,
//     offset: usize,
// }

// impl<I: Io> Append<I> {
//     pub fn new(io: I) -> Self {
//         let data = Data { inner: [1; SIZE] };
//         Self {
//             io,
//             data,
//             offset: 0,
//         }
//     }

//     pub fn write(&mut self) -> io::Result<()> {
//         loop {
//             self.write()
//         }

//         Ok(())
//     }
// }

// trait Io {
//     fn pwrite(&self, buf: &mut [u8]) -> io::Result<()>;
// }
