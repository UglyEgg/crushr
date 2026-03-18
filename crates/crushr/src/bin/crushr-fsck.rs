fn main() {
    eprintln!(
        "crushr-fsck is retired. Use `crushr-extract --verify <archive>` for strict verification, or `crushr-salvage` for recovery-oriented analysis."
    );
    std::process::exit(2);
}
