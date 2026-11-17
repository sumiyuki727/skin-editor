//! Explicit experiment roots only; no source connection or recovery operation.
use super::*;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

pub(super) const PATH_CHARGE: usize = 16 * 1024;
pub(super) const LOCK_NAME: &str = "file-facts-reader.lock";
pub(super) struct RootLock {
    file: Option<File>,
    path: PathBuf,
}
impl RootLock {
    pub fn create(root: &Path) -> Result<Self, FileFactsError> {
        let path = root.join(LOCK_NAME);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0).custom_flags(0x0020_0000);
        }
        let file = options.open(&path).map_err(io_error)?;
        Ok(Self {
            file: Some(file),
            path,
        })
    }
}
impl Drop for RootLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}
pub(super) struct BoundPath {
    pub path: PathBuf,
    directories: [Option<File>; 17],
    _lease: crate::byte_budget::ByteLease,
}
fn io_error(error: std::io::Error) -> FileFactsError {
    if matches!(error.raw_os_error(), Some(32 | 33)) {
        FileFactsError::Busy
    } else {
        FileFactsError::Io
    }
}
pub(super) fn root_empty(root: &Path, own_lock: bool) -> Result<(), FileFactsError> {
    for entry in std::fs::read_dir(root).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        if own_lock && entry.file_name() == LOCK_NAME {
            continue;
        }
        if !own_lock && entry.file_name() == LOCK_NAME {
            // Read-only check distinguishes the live exclusive owner from a
            // leftover marker. Never remove an unowned marker.
            if let Err(FileFactsError::Busy) = open(&entry.path(), false, 3) {
                return Err(FileFactsError::Busy);
            }
        }
        return Err(FileFactsError::Unsupported);
    }
    Ok(())
}
fn regular(file: &File, directory: bool) -> Result<(), FileFactsError> {
    let metadata = file.metadata().map_err(io_error)?;
    if metadata.is_dir() != directory || (!directory && !metadata.is_file()) {
        return Err(FileFactsError::Unsupported);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(FileFactsError::Unsupported);
        }
    }
    Ok(())
}
pub(super) fn open(path: &Path, directory: bool, share: u32) -> Result<File, FileFactsError> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let flags = 0x0020_0000 | if directory { 0x0200_0000 } else { 0 };
        let file = OpenOptions::new()
            .read(true)
            .share_mode(share)
            .custom_flags(flags)
            .open(path)
            .map_err(io_error)?;
        regular(&file, directory)?;
        Ok(file)
    }
    #[cfg(not(windows))]
    {
        let _ = (path, directory, share);
        Err(FileFactsError::Unsupported)
    }
}
pub(super) fn validate_path(path: &Path) -> Result<(), FileFactsError> {
    #[cfg(windows)]
    {
        use std::path::Prefix;
        let text = path.to_str().ok_or(FileFactsError::Unsupported)?;
        if text.encode_utf16().count() > 1024 {
            return Err(FileFactsError::Unsupported);
        }
        let mut parts = path.components();
        match parts.next() {
            Some(Component::Prefix(p))
                if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)) => {}
            _ => return Err(FileFactsError::Unsupported),
        }
        if parts.next() != Some(Component::RootDir) {
            return Err(FileFactsError::Unsupported);
        }
        let mut count = 0;
        for part in parts {
            let Component::Normal(name) = part else {
                return Err(FileFactsError::Unsupported);
            };
            let name = name.to_str().ok_or(FileFactsError::Unsupported)?;
            if name.contains(':') || name.ends_with([' ', '.']) {
                return Err(FileFactsError::Unsupported);
            }
            count += 1;
        }
        if count > 16 {
            return Err(FileFactsError::Unsupported);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(FileFactsError::Unsupported)
    }
}
impl BoundPath {
    pub fn bind(
        path: &Path,
        directory: bool,
        exclusive: bool,
        control: &Arc<crate::byte_budget::ByteBudget>,
    ) -> Result<Self, FileFactsError> {
        validate_path(path)?;
        let lease = control
            .reserve(PATH_CHARGE)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let mut directories = std::array::from_fn(|_| None);
        let mut built = PathBuf::with_capacity(4096);
        let parent = if directory {
            path
        } else {
            path.parent().ok_or(FileFactsError::Unsupported)?
        };
        let mut index = 0;
        for component in parent.components() {
            built.push(component.as_os_str());
            if matches!(component, Component::RootDir | Component::Normal(_)) {
                let share = if exclusive && built == parent { 0 } else { 3 };
                directories[index] = Some(open(&built, true, share)?);
                index += 1;
            }
        }
        let owned = path.to_path_buf();
        if built.capacity() > 4096 || owned.capacity() > 4096 {
            return Err(FileFactsError::Limit(FileFactsLimit::Control));
        }
        Ok(Self {
            path: owned,
            directories,
            _lease: lease,
        })
    }
    pub fn verify(&self) -> Result<(), FileFactsError> {
        for file in self.directories.iter().flatten() {
            regular(file, true)?;
        }
        Ok(())
    }
}
pub(super) fn healthy(path: &Path) -> Result<(), FileFactsError> {
    for suffix in ["-journal", "-wal", "-shm"] {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        match std::fs::symlink_metadata(Path::new(&name)) {
            Ok(_) => return Err(FileFactsError::Unsupported),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(io_error(e)),
        }
    }
    Ok(())
}
pub(super) fn hash(
    file: &mut File,
    bytes: u64,
    cancel: &CaptureCancellation,
) -> Result<[u8; 32], FileFactsError> {
    file.seek(SeekFrom::Start(0)).map_err(io_error)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut left = bytes;
    while left > 0 {
        if cancel.is_cancelled() {
            return Err(FileFactsError::Cancelled);
        }
        let n = left.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..n]).map_err(io_error)?;
        hash.update(&buffer[..n]);
        left -= n as u64;
    }
    if file.read(&mut [0]).map_err(io_error)? != 0 {
        return Err(FileFactsError::TargetConflict);
    }
    Ok(hash.finalize().into())
}
pub(super) struct Source {
    pub file: File,
    pub bound: BoundPath,
    pub bytes: u64,
    pub digest: [u8; 32],
}
impl Source {
    pub fn bind(
        path: &Path,
        limit: u64,
        control: &Arc<crate::byte_budget::ByteBudget>,
        cancel: &CaptureCancellation,
    ) -> Result<Self, FileFactsError> {
        let bound = BoundPath::bind(path, false, false, control)?;
        let mut file = open(path, false, 0)?;
        healthy(path)?;
        let bytes = file.metadata().map_err(io_error)?.len();
        if bytes > limit {
            return Err(FileFactsError::Limit(FileFactsLimit::File));
        }
        let digest = hash(&mut file, bytes, cancel)?;
        bound.verify()?;
        Ok(Self {
            bound,
            file,
            bytes,
            digest,
        })
    }
    pub fn verify(&mut self, cancel: &CaptureCancellation) -> Result<(), FileFactsError> {
        self.bound.verify()?;
        healthy(&self.bound.path)?;
        if hash(&mut self.file, self.bytes, cancel)? != self.digest {
            return Err(FileFactsError::TargetConflict);
        }
        Ok(())
    }
    pub fn copy(
        &mut self,
        to: &mut File,
        cancel: &CaptureCancellation,
    ) -> Result<(), FileFactsError> {
        self.file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut buffer = [0u8; 65536];
        let mut left = self.bytes;
        let mut digest = Sha256::new();
        while left > 0 {
            if cancel.is_cancelled() {
                return Err(FileFactsError::Cancelled);
            }
            let n = left.min(buffer.len() as u64) as usize;
            self.file.read_exact(&mut buffer[..n]).map_err(io_error)?;
            to.write_all(&buffer[..n]).map_err(io_error)?;
            digest.update(&buffer[..n]);
            left -= n as u64;
        }
        if <[u8; 32]>::from(digest.finalize()) != self.digest {
            return Err(FileFactsError::TargetConflict);
        }
        to.flush().map_err(io_error)
    }
}
pub(super) struct Scratch {
    pub path: PathBuf,
    pub token: u128,
    pub bytes: u64,
    identity: Option<same_file::Handle>,
}
impl Scratch {
    pub fn create(root: &Path, bytes: u64) -> Result<(Self, File), FileFactsError> {
        let token = identity()?;
        let path = root.join(format!("file-facts-{token:032x}.scratch"));
        validate_path(&path)?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(1).custom_flags(0x0020_0000);
        }
        let file = options.open(&path).map_err(io_error)?;
        Ok((
            Self {
                path,
                token,
                bytes,
                identity: None,
            },
            file,
        ))
    }
    pub fn bind_identity(&mut self) -> Result<(), FileFactsError> {
        self.identity = Some(same_file::Handle::from_path(&self.path).map_err(io_error)?);
        Ok(())
    }
    pub fn cleanup(&self) -> Result<(), FileFactsError> {
        let identity = self
            .identity
            .as_ref()
            .ok_or(FileFactsError::Cleanup { token: self.token })?;
        let current = same_file::Handle::from_path(&self.path)
            .map_err(|_| FileFactsError::Cleanup { token: self.token })?;
        if &current != identity {
            return Err(FileFactsError::Cleanup { token: self.token });
        }
        std::fs::remove_file(&self.path).map_err(|_| FileFactsError::Cleanup { token: self.token })
    }
}
pub(super) fn identity() -> Result<u128, FileFactsError> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).map_err(|_| FileFactsError::IdentityUnavailable)?;
    let value = u128::from_be_bytes(bytes);
    if value == 0 {
        Err(FileFactsError::IdentityUnavailable)
    } else {
        Ok(value)
    }
}
