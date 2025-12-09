pub const FILE_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];

pub const EXCLUDED_DIRS: &[&str] = &[
    "testdata",
    ".git",
    "node_modules",
    ".next",
    ".nuxt",
    "dist",
    "build",
];

pub const NPM_COMMAND: &str = "npm";

pub const YARN_COMMAND: &str = "yarn";

pub const PNPM_COMMAND: &str = "pnpm";

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
