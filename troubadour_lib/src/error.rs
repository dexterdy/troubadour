use std::io;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
#[error("{msg}")]
pub struct Error {
    pub msg: String,
    pub variant: ErrorVariant,
    pub(crate) source: Option<anyhow::Error>,
}

#[derive(Debug)]
pub enum ErrorVariant {
    AudioDeviceSetupFailed,
    FileNotFound(FileKind),
    PermissionDenied(FileKind),
    GenericIo(FileKind),
    IsADirectory(FileKind),
    FileAlreadyExists(FileKind),
    Serialization,
    Deserialization,
    NoPlayers,
    DecoderFailed,
    NameConflict,
    MissingId,
    MissingGroupId,
    InvalidId,
    InvalidGroupId,
    OperationFailed,
}

#[derive(Debug)]
pub enum FileKind {
    Media,
    Save,
}

pub(crate) fn convert_read_file_error(path: &Path, err: io::Error, kind: FileKind) -> Error {
    let path_dis = path.display();
    match err.kind() {
        io::ErrorKind::NotFound => Error {
            msg: format!("error: could not find a file at {path_dis}."),
            variant: ErrorVariant::FileNotFound(kind),
            source: Some(err.into()),
        },
        io::ErrorKind::PermissionDenied => Error {
            msg: format!("error: permission to access {path_dis} was denied."),
            variant: ErrorVariant::PermissionDenied(kind),
            source: Some(err.into()),
        },
        _ => Error {
            msg: format!("error: something went wrong trying to open {path_dis}. {err}"),
            variant: ErrorVariant::GenericIo(kind),
            source: Some(err.into()),
        },
    }
}

pub(crate) fn convert_write_file_error(path: &Path, err: io::Error, kind: FileKind) -> Error {
    let path_dis = path.display();
    match err.kind() {
        io::ErrorKind::AlreadyExists => Error {
            msg: format!("error: {path_dis} already exists."),
            variant: ErrorVariant::FileAlreadyExists(kind),
            source: Some(err.into()),
        },
        io::ErrorKind::IsADirectory => Error {
            msg: format!("error: {path_dis} is a directory."),
            variant: ErrorVariant::IsADirectory(kind),
            source: Some(err.into()),
        },
        io::ErrorKind::PermissionDenied => Error {
            msg: format!("error: permission to write to {path_dis} was denied."),
            variant: ErrorVariant::PermissionDenied(kind),
            source: Some(err.into()),
        },
        _ => Error {
            msg: format!("error: something went wrong trying to write {path_dis}. {err}"),
            variant: ErrorVariant::GenericIo(kind),
            source: Some(err.into()),
        },
    }
}
