use libc;

pub enum ActionError {
	Conflict(String),
	NoEntry,
	FileExists,
	NotADirectory,
	IsADirectory,
	NoSpaceLeftOnDevice,
	NotImplemented
}

/// Convert to libc error
impl Into<libc::c_int> for ActionError {
	fn into(self) -> libc::c_int {
		match self {
			ActionError::NoEntry             => libc::ENOENT,
			ActionError::FileExists          => libc::EEXIST,
			ActionError::NotADirectory       => libc::ENOTDIR,
			ActionError::IsADirectory        => libc::EISDIR,
			ActionError::NoSpaceLeftOnDevice => libc::ENOSPC,
			ActionError::NotImplemented      => libc::ENOSYS,
			_                                => 0
		}
	}
}

/// Modifications are run as actions
/// Actions are serializable and atomic
pub trait Action {

	/// Return the name
	fn get_name(&self) -> &str;

	/// Run the action
	fn run(&mut self, _replay: bool) -> Result<(), ActionError>;
}

