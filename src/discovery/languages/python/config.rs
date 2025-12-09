pub const FILE_EXTENSIONS: &[&str] = &["py"];

pub const EXCLUDED_DIRS: &[&str] = &["testdata", ".git", "__pycache__", ".pytest_cache"];

pub const PYTHON_COMMAND: &str = "python3";

pub const PIP_COMMAND: &str = "pip3";

pub const POETRY_COMMAND: &str = "poetry";

pub const UV_COMMAND: &str = "uv";

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
