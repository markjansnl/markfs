use std::path::Path;
use error::Error;

pub trait FileHandle {
    fn read(&mut self, _offset: u64, _size: u32) -> Result<Vec<u8>, Error>;
    fn write(&mut self, _offset: u64, _data: Vec<u8>) -> Result<(), Error>;
}

pub trait FileOperation {
    fn create_dir(path: &Path) -> Result<(), Error>;
    fn rename(old_path: &Path, new_path: &Path) -> Result<(), Error>;
    fn open(path: &Path, flags: u32) -> Result<Box<FileHandle>, Error>;
}
