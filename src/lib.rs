use std::path::Path;

#[cfg(feature = "cpp-11")]
const CPP_VERSION: &str = "-std=c++11";
#[cfg(feature = "cpp-14")]
const CPP_VERSION: &str = "-std=c++14";
#[cfg(feature = "cpp-17")]
const CPP_VERSION: &str = "-std=c++17";

pub fn target() -> String {
    std::env::var("TARGET").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetOs {
    Ios(String),
    Android(String),
    MacOs(String),
}

impl TargetOs {
    pub fn detect() -> Option<Self> {
        let target = target();
        if target.contains("ios") {
            Some(Self::Ios(target))
        } else if target.contains("apple") {
            Some(Self::MacOs(target))
        } else if target.contains("android") {
            Some(Self::Android(target))
        } else {
            None
        }
    }

    pub fn is_ios(&self) -> bool {
        match self {
            TargetOs::Ios(_) => true,
            _ => false,
        }
    }
    pub fn is_android(&self) -> bool {
        match self {
            TargetOs::Android(_) => true,
            _ => false,
        }
    }
    pub fn is_macos(&self) -> bool {
        match self {
            TargetOs::MacOs(_) => true,
            _ => false,
        }
    }
}

pub fn sdk_path(target: &str) -> Option<String> {
    let sdk = if target.contains("apple-darwin") {
        "macosx"
    } else if target == "x86_64-apple-ios" || target == "i386-apple-ios" {
        "iphonesimulator"
    } else if target == "aarch64-apple-ios" || target == "armv7-apple-ios" {
        "iphoneos"
    } else {
        return None;
    };

    Some(
        bossy::Command::impure("xcrun")
            .with_args(&["--sdk", sdk, "--show-sdk-path"])
            .run_and_wait_for_str(|s| s.trim().to_string())
            .expect("xcrun command failed"),
    )
}

pub fn default_clang_args(
    includes: &[&str],
    apple_args: &[String],
    android_args: &[String],
) -> Vec<String> {
    let target = target();

    let mut args = vec!["-xc++".into(), "-stdlib=libc++".into(), CPP_VERSION.into()];

    if target.contains("apple") {
        if let Some(sdk_path) = sdk_path(&target) {
            args.push("-isysroot".into());
            args.push(sdk_path);
        }
        apple_args.iter().for_each(|arg| args.push(arg.to_string()));
    }

    if target.contains("android") {
        android_args
            .iter()
            .for_each(|arg| args.push(arg.to_string()));
    }

    // https://github.com/rust-lang/rust-bindgen/issues/1211
    let target = if target == "aarch64-apple-ios" {
        String::from("arm64-apple-ios")
    } else {
        target
    };

    includes
        .iter()
        .for_each(|include| args.push(format!("-I{}", include)));

    args.push(format!("--target={}", target));
    args
}

pub fn recursive_link_dir(link_dir: impl AsRef<Path>, filters: &[&'static str]) {
    let frameworks = walkdir::WalkDir::new(link_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let components = e.path().components();
            for component in components {
                for filter in filters {
                    if *filter == component.as_os_str().to_str().unwrap() {
                        return false;
                    }
                }
            }
            true
        })
        .filter(|dir| {
            dir.path()
                .extension()
                .map_or_else(|| false, |extension| extension == "framework")
        })
        .collect::<Vec<_>>();
    for framework in frameworks {
        let path = framework.path();
        let parent = path.parent().unwrap();
        println!("cargo:rustc-link-search=framework={}", parent.display());
        let framework = path.file_stem().unwrap();
        println!(
            "cargo:rustc-link-lib=framework={}",
            framework.to_str().unwrap()
        );
    }
}
