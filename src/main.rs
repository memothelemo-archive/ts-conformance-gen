use std::{env, process::Command};

static TS_REPO: &str = "https://github.com/microsoft/TypeScript";
static TARGET_VERSION: &str = "v4.7.1";

static CURRENT_HASH_FILE: &str = "./CURRENT_HASH";
static CONFORMANCE_DIST_PATH: &str = "conformance";

type FnReturnType<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn clean() -> FnReturnType {
    eprintln!("Cleaning temp/typescript");

    let delete_task = Command::new("rm")
        .args(&["-rvf", "temp", CONFORMANCE_DIST_PATH, CURRENT_HASH_FILE])
        .status()?;

    if !delete_task.success() {
        Err("failed to delete temp directory")?;
    }

    Ok(())
}

fn generate_current_hash() -> FnReturnType {
    let local_sha = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir("./temp/typescript")
        .output()
        .unwrap();

    let local_sha = String::from_utf8_lossy(&local_sha.stdout).to_string();
    let local_sha = local_sha.trim();

    std::fs::write(CURRENT_HASH_FILE, local_sha)?;
    Ok(())
}

fn update_conformance_tests() -> FnReturnType {
    eprintln!("Updating conformance tests");

    // Downloads the git repository of TypeScript.
    //
    // Because TypeScript repo takes awhile to download the
    // entire thing (including commits), `--depth 1` is added
    let clone_task = Command::new("git")
        .args(&["clone", TS_REPO, "temp/typescript"])
        .status()?;

    if !clone_task.success() {
        Err("failed to clone TypeScript repository")?;
    }

    // Fetches all tags
    let fetch_task = Command::new("git")
        .args(&["fetch", "--all", "--tags", "--prune"])
        .current_dir("./temp/typescript")
        .status()?;

    if !fetch_task.success() {
        Err("failed to fetch everything from TypeScript repository")?;
    }

    // Checkout the target version
    let checkout_task = Command::new("git")
        .args(&[
            "checkout",
            &format!("tags/{}", TARGET_VERSION),
            "-b",
            "__workstation__",
        ])
        .current_dir("./temp/typescript")
        .status()?;

    if !checkout_task.success() {
        Err("failed to checkout repository")?;
    }

    // Getting the local sha commit from recently cloned repository
    generate_current_hash()?;
    Ok(())
}

fn copy_conformance_tests() -> FnReturnType {
    eprintln!("Copying conformance tests");
    let delete_task = Command::new("cp")
        .args(&[
            "-rv",
            "./temp/typescript/tests/cases/conformance",
            CONFORMANCE_DIST_PATH,
        ])
        .status()?;
    if !delete_task.success() {
        Err("failed to copy conformance test files")?;
    }
    Ok(())
}

fn update() -> FnReturnType {
    clean()?;
    update_conformance_tests()?;
    copy_conformance_tests()?;

    Ok(())
}

fn main() -> FnReturnType {
    let mut args = env::args();
    let second_arg = args.nth(1).unwrap_or("".to_string());
    let result = match second_arg.as_str() {
        "clean" => clean(),
        "generate-hash" => generate_current_hash(),
        "force-update" => update_conformance_tests(),
        "copy" => copy_conformance_tests(),
        _ => update(),
    };
    if result.is_ok() {
        eprintln!("Done!");
    } else {
        eprintln!("Failed!");
    }
    result
}
