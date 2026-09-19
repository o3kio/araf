use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=../RELEASE_VERSION");
    for name in ["ARAF_VERSION", "ARAF_GIT_SHA", "ARAF_RELEASE_BUILD"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let canonical = fs::read_to_string("../RELEASE_VERSION").expect("release version authority");
    let canonical = canonical.trim();
    let release = env::var("ARAF_RELEASE_BUILD").as_deref() == Ok("1");
    let version = env::var("ARAF_VERSION").unwrap_or_else(|_| "0.0.0-dev".into());
    let sha = env::var("ARAF_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    if release {
        assert_eq!(
            version, canonical,
            "release version differs from RELEASE_VERSION"
        );
        assert!(
            sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
            "release requires full source SHA"
        );
    }
    println!("cargo:rustc-env=ARAF_BUILD_VERSION={version}");
    println!("cargo:rustc-env=ARAF_BUILD_SHA={sha}");
    println!("cargo:rustc-env=ARAF_BUILD_RELEASE={release}");
}
