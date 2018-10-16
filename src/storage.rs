use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

// General traits
pub trait StorageReadGuard<'a, T>: Deref<Target = T> {}
pub trait StorageWriteGuard<'a, T>: DerefMut<Target = T> {}

pub trait Storage: fmt::Display {
    type S;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, Self::S, Target = Self::S> + 'a>;
    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, Self::S, Target = Self::S> + 'a>;
}

// Disk Storage-related structs and impls
/// TODO: Write some docs
#[derive(Debug)]
pub struct DiskStorageReadGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    __storage: &'a DiskStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> DiskStorageReadGuard<'a, T> {
    fn new(storage: &'a DiskStorage<T>) -> Self {
        Self { __storage: storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for DiskStorageReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for DiskStorageReadGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> StorageReadGuard<'a, T>
    for DiskStorageReadGuard<'a, T>
{}

/// TODO: Write some docs
#[derive(Debug)]
pub struct DiskStorageWriteGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    __storage: &'a mut DiskStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> DiskStorageWriteGuard<'a, T> {
    fn new(storage: &'a mut DiskStorage<T>) -> Self {
        Self { __storage: storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned> Drop for DiskStorageWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.__storage.file.seek(SeekFrom::Start(0)).unwrap();
        self.__storage.file.set_len(0).unwrap();
        self.__storage
            .file
            .write(to_string(&self.__storage.data).unwrap().as_bytes())
            .unwrap();
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for DiskStorageWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__storage.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> DerefMut for DiskStorageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.__storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for DiskStorageWriteGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> StorageWriteGuard<'a, T>
    for DiskStorageWriteGuard<'a, T>
{}

/// TODO: Write some docs
#[derive(Debug)]
pub struct DiskStorage<T: Serialize + DeserializeOwned> {
    data: T,
    file: File,
}

impl<T: Serialize + DeserializeOwned> DiskStorage<T> {
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

                from_str(&contents).map_err(|err| format!("Couldn't read file: {}", err))?
            }
            Err(_) => default(),
        };

        // Then open the file again and truncate, preparing it to be written to
        Ok(Self {
            data,
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(|err| format!("Couldn't open file: {}", err))?,
        })
    }
}

impl<T: fmt::Display + Serialize + DeserializeOwned> fmt::Display for DiskStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).data.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + fmt::Display> Storage for DiskStorage<T> {
    type S = T;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, T, Target = T> + 'a> {
        Box::new(DiskStorageReadGuard::new(self))
    }

    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, T, Target = T> + 'a> {
        Box::new(DiskStorageWriteGuard::new(self))
    }
}

// Mem Storage-related structs and impls
/// TODO: Write some docs
#[derive(Debug)]
pub struct MemStorageReadGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    __storage: &'a MemStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> MemStorageReadGuard<'a, T> {
    fn new(storage: &'a MemStorage<T>) -> Self {
        Self { __storage: storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for MemStorageReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for MemStorageReadGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> StorageReadGuard<'a, T>
    for MemStorageReadGuard<'a, T>
{}

/// TODO: Write some docs
#[derive(Debug)]
pub struct MemStorageWriteGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    __storage: &'a mut MemStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> MemStorageWriteGuard<'a, T> {
    fn new(storage: &'a mut MemStorage<T>) -> Self {
        Self { __storage: storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for MemStorageWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__storage.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> DerefMut for MemStorageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.__storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for MemStorageWriteGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> StorageWriteGuard<'a, T>
    for MemStorageWriteGuard<'a, T>
{}

/// TODO: Write some docs
#[derive(Debug)]
pub struct MemStorage<T: Serialize + DeserializeOwned> {
    data: T,
}

impl<T: Serialize + DeserializeOwned> MemStorage<T> {
    pub fn new_with_default<F: Fn() -> T>(default: F) -> Result<Self, String> {
        Ok(Self { data: default() })
    }
}

impl<T: fmt::Display + Serialize + DeserializeOwned> fmt::Display for MemStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).data.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + fmt::Display> Storage for MemStorage<T> {
    type S = T;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, T, Target = T> + 'a> {
        Box::new(MemStorageReadGuard::new(self))
    }

    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, T, Target = T> + 'a> {
        Box::new(MemStorageWriteGuard::new(self))
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::distributions::Alphanumeric;
    use self::rand::{thread_rng, Rng};
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_read_guard() {
        let filename = String::from("/tmp/") + &thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        let storage = DiskStorage::new_with_default(&filename[..], || 1).unwrap();
        let val = storage.read();
        let other = storage.read();
        assert_eq!(**val, 1);
        assert_eq!(**other, 1);

        remove_file(filename).unwrap();
    }

    #[test]
    fn test_write_guard() {
        let filename = String::from("/tmp/") + &thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        {
            let mut storage = DiskStorage::new_with_default(&filename[..], || 1).unwrap();
            let mut val = storage.write();
            assert_eq!(**val, 1);
            **val = 5;
            assert_eq!(**val, 5);
        }

        {
            let mut storage = DiskStorage::new_with_default(&filename[..], || 1).unwrap();
            let mut val = storage.write();
            assert_eq!(**val, 5);
            **val = 64;
            assert_eq!(**val, 64);
        }

        remove_file(filename).unwrap();
    }

    // The common use case, of passing in a guarded reference
    fn add_refs(foo: &mut u32, bar: &u32) {
        *foo += bar;
    }

    // You can also pass in the storages themselves, if you help the compiler out
    // with specifying lifetimes
    fn add_storages<'a>(
        foo: &'a mut (Storage<'a, S = u32> + 'a),
        bar: &'a mut (Storage<'a, S = u32> + 'a),
    ) {
        **foo.write() += **bar.read();
    }

    #[test]
    fn test_fn_arg() {
        let filename = String::from("/tmp/") + &thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        let mut diskval = DiskStorage::new_with_default(&filename[..], || 1).unwrap();
        let mut memval = MemStorage::new_with_default(|| 5).unwrap();

        assert_eq!(**diskval.read(), 1);
        add_refs(&mut *diskval.write(), &*memval.read());
        assert_eq!(**diskval.read(), 6);

        assert_eq!(**memval.read(), 5);
        add_storages(&mut memval, &mut diskval);
        assert_eq!(**memval.read(), 11);

        remove_file(filename).unwrap();
    }
}
