#[cfg(feature = "cpp-11")]
const CPP_VERSION: &str = "-std=c++11";
#[cfg(feature = "cpp-14")]
const CPP_VERSION: &str = "-std=c++14";
#[cfg(feature = "cpp-17")]
const CPP_VERSION: &str = "-std=c++17";

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
    apple_args: &[&str],
    android_args: &[&str],
) -> Vec<String> {
    let target = std::env::var("TARGET").unwrap();

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
