use crate::hash::Hash;
use crate::storage::Storage;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Error, Seek, SeekFrom, Write};
use std::ptr::hash;

pub struct FileWriter<'a> {
    id: usize,
    storage: &'a Storage,
    hashes: Vec<Hash>,
    size: usize,
    temp_file: File,
}

pub struct FileReader<'a> {
    storage: &'a Storage,
    index: usize,
    hashes: Vec<Hash>,
}

impl<'a> FileWriter<'a> {
    pub(crate) fn new(storage: &'a Storage) -> Self {
        Self {
            id: storage.get_new_id(),
            storage,
            hashes: vec![],
            size: 0,
            temp_file: tempfile::tempfile().expect("couldn't create temp file"),
        }
    }

    fn save(&mut self) {
        let mut hasher = Sha256::new();
        self.temp_file.seek(SeekFrom::Start(0));
        std::io::copy(&mut self.temp_file, &mut hasher);
        let hash_str: String = hasher
            .result()
            .iter()
            .map(|v| format!("{:02x}", v))
            .collect::<Vec<_>>()
            .join("");
        dbg!(&hash_str);

        self.temp_file.seek(SeekFrom::Start(0));
        {
            let hashes_lock = self.storage.get_hashes();
            {
                let mut hashes = hashes_lock.read().expect("could not get write lock");
                if hashes.get(&hash_str).is_some() {
                    dbg!("hash found");
                    return;
                }
            }

            let mut hashes = hashes_lock.write().expect("could not get write lock");

            let val = self.storage.get_storage_file();
            let mut files = val.lock().expect("mutex error!");
            let file_size = files.storage_file.metadata().unwrap().len() as usize;
            std::io::copy(&mut self.temp_file, &mut files.storage_file);
            let new_file_size = files.storage_file.metadata().unwrap().len() as usize;

            writeln!(&mut files.meta, "{} {}", &hash_str, new_file_size);

            hashes.insert(hash_str, (file_size, new_file_size));
        }
    }
}

impl<'a> Write for FileWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        //dbg!(buf);
        if buf.len() < Storage::BLOCK_SIZE - self.size {
            self.temp_file.write(&buf)?;
            self.size += buf.len();
        } else {
            let mut size_left = buf.len();
            let mut size_offset = 0;
            loop {
                if size_left - size_offset < Storage::BLOCK_SIZE - self.size {
                    self.temp_file.write(&buf[size_offset..size_left])?;
                    self.size = size_left - size_offset;
                    break;
                } else {
                    self.temp_file
                        .write(&buf[size_offset..size_offset + Storage::BLOCK_SIZE - self.size])?;
                    //size_left -= Storage::BLOCK_SIZE - self.size;
                    size_offset += Storage::BLOCK_SIZE - self.size;
                    self.save();
                    self.size = 0;
                    dbg!("new file");
                    self.temp_file = tempfile::tempfile()?;
                }
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Drop for FileWriter<'_> {
    fn drop(&mut self) {
        self.save()
    }
}
