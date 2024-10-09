use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=MIGRATIONS_FOLDER");

    let var = env::var("MIGRATIONS_FOLDER").unwrap_or_else(|_| "./migrations".to_string());

    println!("cargo:rustc-env=MIGRATIONS_FOLDER={}", var);
    // Rebuild if i18n files change
    println!("cargo:rerun-if-changed=i18n");

    // Emit version information (if not cached by just vendor)
    let mut vergen = vergen::EmitBuilder::builder();
    println!("cargo:rerun-if-env-changed=VERGEN_GIT_COMMIT_DATE");
    if std::env::var_os("VERGEN_GIT_COMMIT_DATE").is_none() {
        vergen.git_commit_date();
    }
    println!("cargo:rerun-if-env-changed=VERGEN_GIT_SHA");
    if std::env::var_os("VERGEN_GIT_SHA").is_none() {
        vergen.git_sha(false);
    }
    vergen.fail_on_error().emit()?;
    Ok(())
}
