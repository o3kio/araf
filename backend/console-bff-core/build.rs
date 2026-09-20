use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=ARAF_VERSION");
    println!("cargo:rerun-if-env-changed=ARAF_GIT_SHA");
    println!("cargo:rerun-if-env-changed=ARAF_RELEASE_BUILD");

    let version = env::var("ARAF_VERSION").unwrap_or_else(|_| "0.0.0-dev".to_owned());
    let git_sha = env::var("ARAF_GIT_SHA").unwrap_or_else(|_| "unknown".to_owned());
    let release_build = env::var("ARAF_RELEASE_BUILD").is_ok_and(|value| value == "1");

    if release_build {
        assert!(
            version
                .strip_prefix('v')
                .is_some_and(|value| !value.is_empty()
                    && value
                        .chars()
                        .all(|c| { c.is_ascii_alphanumeric() || matches!(c, '.' | '-') })),
            "ARAF_VERSION must be a non-empty v-prefixed release version"
        );
        assert!(
            git_sha.len() == 40 && git_sha.chars().all(|c| c.is_ascii_hexdigit()),
            "ARAF_GIT_SHA must be a 40-character hexadecimal source SHA for release builds"
        );
    }

    println!("cargo:rustc-env=ARAF_BUILD_VERSION={version}");
    println!("cargo:rustc-env=ARAF_BUILD_GIT_SHA={git_sha}");
}
