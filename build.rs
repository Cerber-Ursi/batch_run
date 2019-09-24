use std::{env, fs::File, io::Write, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR not set, check if your Cargo version isn't really ancient");
    let dest_path = Path::new(&out_dir).join("info.rs");
    let mut f = File::create(&dest_path).expect("Unable to create file in OUT_DIR");

    f.write_all(
        format!(
            "
mod info {{
    pub fn opt_level() -> &'static str {{
        \"{}\"
    }}

    pub fn rustc() -> &'static str {{
        \"{}\"
    }}
}}
    ",
            env::var("PROFILE")
                .expect("PROFILE not set, check if your Cargo version isn't too old"),
            env::var("RUSTC").expect("RUSTC not set, check if your Cargo version isn't too old"),
        )
        .as_bytes(),
    )
    .expect("Unable to write to OUT_DIR");
}
