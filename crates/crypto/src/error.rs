//! This module contains all possible errors that this crate can return.

use std::string::FromUtf8Error;

use thiserror::Error;

#[cfg(feature = "rspc")]
impl From<Error> for rspc::Error {
	fn from(err: Error) -> Self {
		Self::new(rspc::ErrorCode::InternalServerError, err.to_string())
	}
}

#[cfg(feature = "headers")]
impl From<Error> for bincode::error::EncodeError {
	fn from(value: Error) -> Self {
		Self::OtherString(value.to_string())
	}
}

#[cfg(feature = "headers")]
impl From<Error> for bincode::error::DecodeError {
	fn from(value: Error) -> Self {
		Self::OtherString(value.to_string())
	}
}

pub type Result<T> = std::result::Result<T, Error>;

/// This enum defines all possible errors that this crate can give
#[derive(Error, Debug)]
pub enum Error {
	// crypto primitive errors (STREAM, hashing)
	#[error("there was an error while password hashing")]
	PasswordHash,
	#[error("error while encrypting")]
	Encrypt,
	#[error("error while decrypting")]
	Decrypt,
	#[error("nonce length mismatch")]
	NonceLengthMismatch,
	#[error("error initialising stream encryption/decryption")]
	StreamModeInit,

	// header errors
	#[cfg(feature = "headers")]
	#[error("no keyslots available")]
	NoKeyslots,
	#[cfg(feature = "headers")]
	#[error("tried adding too many keyslots to a header")]
	TooManyKeyslots,
	#[cfg(feature = "headers")]
	#[error("no header objects available (or none that match)")]
	NoObjects,
	#[cfg(feature = "headers")]
	#[error("tried to run an object operation which resulted in duplicates")]
	DuplicateObjects,
	#[cfg(feature = "headers")]
	#[error("tried adding too many objects to a header")]
	TooManyObjects,
	#[cfg(feature = "headers")]
	#[error("error while encoding with bincode: {0}")]
	BincodeEncode(#[from] bincode::error::EncodeError),
	#[cfg(feature = "headers")]
	#[error("error while decoding with bincode: {0}")]
	BincodeDecode(#[from] bincode::error::DecodeError),

	// key manager
	#[error("requested key wasn't found in the key manager")]
	KeyNotFound,
	#[error("key is already mounted")]
	KeyAlreadyMounted,
	#[error("key not mounted")]
	KeyNotMounted,
	#[error("key isn't in the queue")]
	KeyNotQueued,
	#[error("key is already in the queue")]
	KeyAlreadyQueued,
	#[error("no default key has been set")]
	NoDefaultKeySet,
	#[error("keymanager is not unlocked")]
	NotUnlocked,
	#[error("no verification key")]
	NoVerificationKey,
	#[error("key isn't flagged as memory only")]
	KeyNotMemoryOnly,

	// general errors
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("mismatched data length while converting vec to array")]
	VecArrSizeMismatch,
	#[error("incorrect password/details were provided")]
	IncorrectPassword,
	#[error("error while serializing/deserializing an item")]
	Serialization,
	#[error("string parse error")]
	StringParse(#[from] FromUtf8Error),

	// keyring
	#[cfg(all(target_os = "linux", feature = "os-keyrings"))]
	#[error("error with the linux keyring: {0}")]
	LinuxKeyringError(#[from] secret_service::Error),
	#[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "os-keyrings"))]
	#[error("error with the apple keyring: {0}")]
	AppleKeyringError(#[from] security_framework::base::Error),
	#[cfg(feature = "os-keyrings")]
	#[error("generic keyring error")]
	KeyringError,
	#[cfg(feature = "os-keyrings")]
	#[error("keyring not available on this platform")]
	KeyringNotSupported,
}
