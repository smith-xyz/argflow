pub const FILE_EXTENSIONS: &[&str] = &["go"];

pub const EXCLUDED_DIRS: &[&str] = &["testdata", ".git"];

pub const GO_COMMAND: &str = "go";

pub const STD_PREFIX: &str = "std/";
pub const INTERNAL_PREFIX: &str = "internal/";

pub const GO_LIST_STD_ARGS: &[&str] = &["list", "std"];

pub const GO_LIST_DEPS_ARGS: &[&str] = &["list", "-deps", "-f"];
pub const GO_LIST_IMPORT_PATH_TEMPLATE: &str = "{{.ImportPath}}";
pub const GO_LIST_PACKAGE_PATTERN: &str = "./...";

pub const GO_LIST_DIR_ARGS: &[&str] = &["list", "-f"];
pub const GO_LIST_DIR_TEMPLATE: &str = "{{.Dir}}";

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
