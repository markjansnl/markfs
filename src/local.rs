use std;
use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::SeekFrom;
use std::io::prelude::*;
//use libc::O_RDONLY;

use markfs::FileHandle;

pub struct LocalFileHandle {
	file: File
}

impl LocalFileHandle {
	pub fn new(path: &Path, _flags: i32) -> LocalFileHandle {
		let mut options = OpenOptions::new();

		//if flags & O_RDONLY == O_RDONLY {
			options.read(true);
		//}

		LocalFileHandle {
			file: options.open(path).unwrap()
		}
	}
}

impl FileHandle for LocalFileHandle {
	fn read(&mut self, offset: u64, size: u32) -> Result<Vec<u8>, ()> {
        self.file.seek(SeekFrom::Start(offset)).unwrap();

        let mut data = Vec::<u8>::with_capacity(size as usize);
        unsafe { data.set_len(size as usize) };

        match self.file.read(&mut data) {
        	Ok(n) => {
        		data.truncate(n);
        		Ok(data)
        	},
        	Err(_e) => Err(())
        }
	}
}

pub struct LocalFileOperations;

impl LocalFileOperations {
	pub fn rename(old_path: &Path, new_path: &Path) -> Result<(), ()> {
		match std::fs::rename(old_path, new_path) {
			Ok(_)  => Ok(()),
			Err(_) => Err(())
		}
	}
}