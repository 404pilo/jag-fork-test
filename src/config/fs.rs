//! A small trait over the file system, so tests give fakes (decision 17).

use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::Path;

/// An open file. The mode comes from the open handle (`fstat`), so no other
/// file can take its place between the check and the read.
pub trait OpenFile {
    fn mode(&self) -> io::Result<u32>;
    fn read_to_string(&mut self) -> io::Result<String>;
}

pub trait Fs {
    /// Open a file for reading. This follows a symbolic link (decision 20).
    fn open(&self, path: &Path) -> io::Result<Box<dyn OpenFile>>;
    /// True when the path is a symbolic link. A missing path is not a link.
    fn is_symlink(&self, path: &Path) -> io::Result<bool>;
    /// True when the path exists and is not a regular file, after jag follows
    /// each link (a folder or a pipe, for example). A missing path is not special.
    fn is_special(&self, path: &Path) -> io::Result<bool>;
    /// Create a folder and its parents with the default mode.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    /// Create one folder with mode `0o700`. A folder that exists stays as it is.
    fn create_private_dir(&self, path: &Path) -> io::Result<()>;
    /// Create a new file with mode `0o600`, write the text, and sync it.
    fn create_private_file(&self, path: &Path, text: &str) -> io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

/// The production file system.
pub struct RealFs;

struct RealOpenFile(File);

impl OpenFile for RealOpenFile {
    fn mode(&self) -> io::Result<u32> {
        Ok(self.0.metadata()?.mode())
    }

    fn read_to_string(&mut self) -> io::Result<String> {
        let mut text = String::new();
        self.0.read_to_string(&mut text)?;
        Ok(text)
    }
}

impl Fs for RealFs {
    fn open(&self, path: &Path) -> io::Result<Box<dyn OpenFile>> {
        Ok(Box::new(RealOpenFile(File::open(path)?)))
    }

    fn is_symlink(&self, path: &Path) -> io::Result<bool> {
        match fs::symlink_metadata(path) {
            Ok(metadata) => Ok(metadata.file_type().is_symlink()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn is_special(&self, path: &Path) -> io::Result<bool> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(!metadata.file_type().is_file()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn create_private_dir(&self, path: &Path) -> io::Result<()> {
        match DirBuilder::new().mode(0o700).create(path) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            other => other,
        }
    }

    fn create_private_file(&self, path: &Path, text: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}
