use super::*;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

pub(super) fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
fn regular(path: &Path) -> Result<(), StorageError> {
    let m = std::fs::symlink_metadata(path).map_err(io)?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err(StorageError::Unsupported(
            "regular recovery source required",
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(StorageError::Unsupported("reparse recovery source"));
        }
    }
    Ok(())
}
fn exclusive(path: &Path) -> Result<File, StorageError> {
    regular(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(path)
            .map_err(|e| {
                if matches!(e.raw_os_error(), Some(32 | 33)) {
                    StorageError::Busy
                } else {
                    io(e)
                }
            })
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(StorageError::Unsupported(
            "exclusive recovery snapshot on this platform",
        ))
    }
}
pub(super) fn absent(path: &Path) -> Result<(), StorageError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Err(StorageError::TargetExists),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(e)),
    }
}
pub(super) fn new_target(path: &Path) -> Result<(), StorageError> {
    for suffix in ["", "-journal", "-wal", "-shm"] {
        absent(&sibling(path, suffix))?;
    }
    Ok(())
}
pub(super) fn separate_target(source: &Path, target: &Path) -> Result<(), StorageError> {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(StorageError::Unsupported(
            "non-Unicode recovery target name",
        ))?
        .trim_end_matches([' ', '.']);
    for suffix in ["-journal", "-wal", "-shm"] {
        if let Some(cut) = name.len().checked_sub(suffix.len())
            && let Some(tail) = name.get(cut..)
            && tail.eq_ignore_ascii_case(suffix)
            && cut > 0
        {
            // Resolve the existing prefix through the filesystem. This also
            // catches case variants, directory aliases and hard-link aliases.
            let prefix = target.with_file_name(&name[..cut]);
            match same_file::is_same_file(source, &prefix) {
                Ok(true) => {
                    return Err(StorageError::Invalid(
                        "recovery target overlaps source sidecars",
                    ));
                }
                Ok(false) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(e) => return Err(io(e)),
            }
        }
    }
    Ok(())
}
pub(super) fn digest(
    file: &mut File,
    limit: u64,
    cancel: &CaptureCancellation,
) -> Result<RecoveryDigest, StorageError> {
    file.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut hash = Sha256::new();
    let mut len = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        check(cancel)?;
        let n = file.read(&mut buffer).map_err(io)?;
        if n == 0 {
            break;
        }
        len = len
            .checked_add(n as u64)
            .ok_or(StorageError::Limit("recovery source"))?;
        if len > limit {
            return Err(StorageError::Limit("recovery source"));
        }
        hash.update(&buffer[..n]);
    }
    Ok(RecoveryDigest {
        byte_len: len,
        sha256: hash.finalize().into(),
    })
}
pub(super) struct Part {
    pub file: File,
    pub digest: RecoveryDigest,
    modified: std::time::SystemTime,
}
impl Part {
    fn open(path: &Path, limit: u64, cancel: &CaptureCancellation) -> Result<Self, StorageError> {
        let mut file = exclusive(path)?;
        let m = file.metadata().map_err(io)?;
        if m.len() > limit {
            return Err(StorageError::Limit("recovery source"));
        }
        let found = digest(&mut file, limit, cancel)?;
        if found.byte_len != m.len() {
            return Err(StorageError::ExternalChange);
        }
        Ok(Self {
            file,
            digest: found,
            modified: m.modified().map_err(io)?,
        })
    }
    pub fn copy_into(
        &mut self,
        output: &mut File,
        cancel: &CaptureCancellation,
    ) -> Result<(), StorageError> {
        self.file.seek(SeekFrom::Start(0)).map_err(io)?;
        let mut hash = Sha256::new();
        let mut len = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            check(cancel)?;
            let n = self.file.read(&mut buffer).map_err(io)?;
            if n == 0 {
                break;
            }
            len = len
                .checked_add(n as u64)
                .ok_or(StorageError::ExternalChange)?;
            if len > self.digest.byte_len {
                return Err(StorageError::ExternalChange);
            }
            hash.update(&buffer[..n]);
            output.write_all(&buffer[..n]).map_err(io)?;
        }
        if (RecoveryDigest {
            byte_len: len,
            sha256: hash.finalize().into(),
        }) != self.digest
        {
            return Err(StorageError::ExternalChange);
        }
        output.flush().map_err(io)
    }
    fn verify(&mut self, cancel: &CaptureCancellation) -> Result<(), StorageError> {
        let m = self.file.metadata().map_err(io)?;
        if m.modified().map_err(io)? != self.modified
            || digest(&mut self.file, self.digest.byte_len, cancel)? != self.digest
        {
            return Err(StorageError::ExternalChange);
        }
        Ok(())
    }
}
/// All source handles are read-only. The Windows sharing lease also excludes
/// existing SQLite readers/writers. Other platforms fail closed until their
/// snapshot primitive is verified; flock alone does not match SQLite's locks.
pub(super) struct Frozen {
    pub main: Part,
    pub journal: Option<Part>,
    path: PathBuf,
}
impl Frozen {
    pub fn open(
        path: &Path,
        limits: FileLimits,
        cancel: &CaptureCancellation,
    ) -> Result<Self, StorageError> {
        let mut main = Part::open(path, limits.file_bytes, cancel)?;
        for suffix in ["-wal", "-shm"] {
            if let Err(e) = absent(&sibling(path, suffix)) {
                return Err(if e == StorageError::TargetExists {
                    StorageError::Unsupported("WAL recovery")
                } else {
                    e
                });
            }
        }
        let mut header = [0u8; 100];
        main.file.seek(SeekFrom::Start(0)).map_err(io)?;
        main.file.read_exact(&mut header).map_err(io)?;
        if &header[..16] != b"SQLite format 3\0"
            || header[16..20] != [0x10, 0, 1, 1]
            || u32::from_be_bytes(header[68..72].try_into().expect("fixed")) != APP_ID
        {
            return Err(StorageError::Unsupported("recovery database header"));
        }
        let journal_path = sibling(path, "-journal");
        let cap = limits
            .file_bytes
            .checked_mul(2)
            .and_then(|n| n.checked_add(1024 * 1024))
            .ok_or(StorageError::Limit("journal size"))?;
        let journal = match std::fs::symlink_metadata(&journal_path) {
            Ok(_) => Some(Part::open(&journal_path, cap, cancel)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(io(e)),
        };
        Ok(Self {
            main,
            journal,
            path: path.to_path_buf(),
        })
    }
    pub fn verify(&mut self, cancel: &CaptureCancellation) -> Result<(), StorageError> {
        for suffix in ["-wal", "-shm"] {
            absent(&sibling(&self.path, suffix)).map_err(|_| StorageError::ExternalChange)?;
        }
        if self.journal.is_none() {
            absent(&sibling(&self.path, "-journal")).map_err(|_| StorageError::ExternalChange)?;
        }
        self.main.verify(cancel)?;
        if let Some(j) = &mut self.journal {
            j.verify(cancel)?;
        }
        Ok(())
    }
}

pub(super) struct Scratch {
    pub path: PathBuf,
    pub token: u128,
    journal_owned: bool,
    retain: bool,
}
impl Scratch {
    pub fn create(target: &Path) -> Result<Self, StorageError> {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).map_err(|_| StorageError::Io)?;
        let name = target
            .file_name()
            .ok_or(StorageError::Invalid("target filename"))?
            .to_string_lossy();
        let path = target.with_file_name(format!(".{name}.recover-{}.tmp", hex(&nonce)));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(io)?;
        Ok(Self {
            path,
            token: u128::from_be_bytes(nonce),
            journal_owned: false,
            retain: false,
        })
    }
    pub fn copy(
        &mut self,
        source: &mut Frozen,
        cancel: &CaptureCancellation,
        probe: Probe,
    ) -> Result<(), StorageError> {
        let mut out = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(io)?;
        source.main.copy_into(&mut out, cancel)?;
        drop(out);
        probe.step(Stage::CopyMain, cancel)?;
        if let Some(j) = &mut source.journal {
            let mut out = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(sibling(&self.path, "-journal"))
                .map_err(io)?;
            self.journal_owned = true;
            j.copy_into(&mut out, cancel)?;
            drop(out);
        }
        probe.step(Stage::CopyJournal, cancel)
    }
    pub fn cleanup(&mut self, probe: Probe) -> Result<(), StorageError> {
        #[cfg(test)]
        if probe.fail_cleanup {
            self.retain = true;
            return Err(StorageError::Io);
        }
        let _ = probe;
        let mut failed = false;
        for (path, owned) in [
            (sibling(&self.path, "-journal"), self.journal_owned),
            (self.path.clone(), true),
        ] {
            if owned {
                match std::fs::remove_file(path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(_) => failed = true,
                }
            }
        }
        self.retain = true;
        if failed {
            Err(StorageError::Io)
        } else {
            Ok(())
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.retain {
            let _ = self.cleanup(Probe::default());
        }
    }
}
