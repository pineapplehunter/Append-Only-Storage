use crate::file::FileWriter;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::PathBuf;
use std::ptr::hash;
use std::sync::{Arc, Mutex, RwLock};

pub struct Storage {
    child_count: Arc<Mutex<usize>>,
    storage_file: Arc<Mutex<StorageFiles>>,
    hashes: Arc<RwLock<HashMap<String, (usize, usize)>>>,
}

pub struct StorageFiles {
    pub storage_file: File,
    pub meta: File,
}

impl Storage {
    pub const BLOCK_SIZE: usize = 1024 * 1024;

    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        std::fs::DirBuilder::new().create(&path);
        let storage_file_path = &path.join("storage");
        let storage_meta_path = &path.join("meta");

        let storage_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(storage_file_path)
            .expect("couldn't open storage file");

        let mut buf = String::new();
        {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&storage_meta_path)
                .expect("could not create file");
            let mut storage_meta_file_read = OpenOptions::new()
                .read(true)
                .open(&storage_meta_path)
                .expect("couldn't open storage file");
            storage_meta_file_read
                .read_to_string(&mut buf)
                .expect("???");
        }

        let mut hash_list = HashMap::new();
        let mut prev_size = 0;

        for line in buf.split('\n').filter(|s| s.len() != 0) {
            let v: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
            let size = v[1].parse().unwrap();
            hash_list.insert(v[0].clone(), (prev_size, size));
            prev_size = size;
        }

        //dbg!(&hash_list);
        let storage_meta_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&storage_meta_path)
            .expect("???");

        Self {
            storage_file: Arc::new(Mutex::new(StorageFiles {
                storage_file,
                meta: storage_meta_file,
            })),
            child_count: Arc::new(Mutex::new(0)),
            hashes: Arc::new(RwLock::new(hash_list)),
        }
    }

    pub fn new_file_writer(&self) -> FileWriter {
        FileWriter::new(&self)
    }

    pub fn get_new_id(&self) -> usize {
        let mut count = self.child_count.lock().expect("mutex broke");
        *count += 1;
        *count
    }

    pub fn get_storage_file(&self) -> Arc<Mutex<StorageFiles>> {
        self.storage_file.clone()
    }

    pub fn get_hashes(&self) -> Arc<RwLock<HashMap<String, (usize, usize)>>> {
        self.hashes.clone()
    }
}
