use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::ops::{Deref, DerefMut};
use std::fmt;

/// TODO: Write some docs
#[derive(Debug)]
pub struct StorageReadGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    cache: &'a Storage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> StorageReadGuard<'a, T> {
    fn new(cache: &'a Storage<T>) -> Self {
        Self { cache }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for StorageReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.cache.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display for StorageReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// TODO: Write some docs
#[derive(Debug)]
pub struct StorageWriteGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    cache: &'a mut Storage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> StorageWriteGuard<'a, T> {
    fn new(cache: &'a mut Storage<T>) -> Self {
        Self { cache }
    }
}

impl<'a, T: Serialize + DeserializeOwned> Drop for StorageWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.cache.file.seek(SeekFrom::Start(0)).unwrap();
        self.cache.file.set_len(0).unwrap();
        self.cache.file.write(to_string(&self.cache.data).unwrap().as_bytes()).unwrap();
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for StorageWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.cache.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> DerefMut for StorageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.cache.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display for StorageWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// TODO: Write some docs
#[derive(Debug)]
pub struct Storage<T: Serialize + DeserializeOwned> {
    data: T,
    file: File,
}

impl<T: Serialize + DeserializeOwned> Storage<T> {
    pub fn new_with_default<P: Into<String>, F: Fn() -> T>(
        path: P,
        default: F,
    ) -> Result<Self, String> {
        let path = path.into();

        // Read the file first, to see if there's any existing data
        let data = match File::open(path.clone()) {
            Ok(mut f) => {
                let mut contents = String::new();

                f.read_to_string(&mut contents)
                    .map_err(|err| format!("Couldn't read file: {}", err))?;

                from_str(&contents)
                    .map_err(|err| format!("Couldn't read file: {}", err))?
            }
            Err(_) => default()
        };

        // Then open the file again and truncate, preparing it to be written to
        Ok(Self { data, file: OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(|err| format!("Couldn't open file: {}", err))? })
    }
}

impl<T: Serialize + DeserializeOwned> Storage<T> {
    pub fn read(&self) -> StorageReadGuard<T> {
        StorageReadGuard::new(self)
    }

    pub fn write(&mut self) -> StorageWriteGuard<T> {
        StorageWriteGuard::new(self)
    }
}

impl<T: Serialize + DeserializeOwned + fmt::Display> fmt::Display for Storage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).data.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use self::rand::{Rng, thread_rng};
    use self::rand::distributions::Alphanumeric;
    use std::fs::remove_file;


    #[test]
    fn test_read_guard() {
        let filename = String::from("/tmp/") + &thread_rng().sample_iter(&Alphanumeric).take(10).collect::<String>();

        let cache = Storage::new_with_default(&filename[..], || 1).unwrap();
        let val = cache.read();
        let other = cache.read();
        assert_eq!(val.cache.data, 1);
        assert_eq!(*val, 1);
        assert_eq!(other.cache.data, 1);
        assert_eq!(*other, 1);

        remove_file(filename).unwrap();
    }

    #[test]
    fn test_write_guard() {
        let filename = String::from("/tmp/") + &thread_rng().sample_iter(&Alphanumeric).take(10).collect::<String>();

        {
            let mut cache = Storage::new_with_default(&filename[..], || 1).unwrap();
            let mut val = cache.write();
            assert_eq!(val.cache.data, 1);
            assert_eq!(*val, 1);
            *val = 5;
            assert_eq!(val.cache.data, 5);
            assert_eq!(*val, 5);
        }

        {
            let mut cache = Storage::new_with_default(&filename[..], || 1).unwrap();
            let mut val = cache.write();
            assert_eq!(val.cache.data, 5);
            *val = 64;
            assert_eq!(val.cache.data, 64);
        }

        remove_file(filename).unwrap();
    }

    #[test]
    fn test_display() {
        let filename = String::from("/tmp/") + &thread_rng().sample_iter(&Alphanumeric).take(10).collect::<String>();

        let mut cache = Storage::new_with_default(&filename[..], || 1).unwrap();
        println!("Cache: {}", cache);
        {
            let read = cache.read();
            println!("CacheReadGuard: {}", read);
        }
        let write = cache.write();
        println!("CacheWriteGuard: {}", write);

        remove_file(filename).unwrap();
    }
}
