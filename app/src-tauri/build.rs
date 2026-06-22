fn main() {
    tauri_build::build();

    // Bake public OAuth client IDs into the binary so the installed app works
    // without a .env file in its CWD. .env.secret can still override at runtime.
    let _ = dotenvy::dotenv();
    for key in &[
        "MLMAIL_GOOGLE_DESKTOP_CLIENT_ID",
        "MLMAIL_GOOGLE_ANDROID_CLIENT_ID",
        "MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID",
    ] {
        if let Ok(val) = std::env::var(key) {
            println!("cargo:rustc-env={key}={val}");
        }
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=.env");
}
