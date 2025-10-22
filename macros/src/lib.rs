use proc_macro::TokenStream;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[proc_macro]
pub fn gbasm(input: TokenStream) -> TokenStream {
    let src = input.to_string();
    let trimmed = src.trim();
    let lines = trimmed
        .trim_start_matches('r')
        .trim_matches(&['#', '"'][..])
        .replace("        ", "  ") // ← replace 8-space indents with 2-space indents
        .to_string();
    let mut path = env::temp_dir();
    path.push(format!("{}-{}", "gbasm", uuid::Uuid::new_v4()));
    let tmp_dir = PathBuf::from(&path);
    fs::create_dir(&tmp_dir).expect("failed to create tmp dir");

    let asm_path = tmp_dir.join("macro.asm");
    let obj_path = tmp_dir.join("macro.o");
    let gb_path = tmp_dir.join("macro.gb");
    let asm_str = asm_path.to_str().unwrap();
    let obj_str = obj_path.to_str().unwrap();
    let gb_str = gb_path.to_str().unwrap();

    let full_asm = format!(
        r#"SECTION "Header", ROM0[$100]
  jp EntryPoint
  ds $150 - @, 0

EntryPoint:{}
  halt
"#,
        lines
    );
    fs::write(&asm_path, full_asm).expect("failed to write temp asm");

    let status = Command::new("rgbasm")
        .args(["-o", obj_str, asm_str])
        .status()
        .expect("failed to run rgbasm");
    assert!(status.success(), "rgbasm failed");

    let status = Command::new("rgblink")
        .args(["-o", gb_str, obj_str])
        .status()
        .expect("failed to run rgblink");
    assert!(status.success(), "rgblink failed");

    let mut bytes = fs::read(&gb_path).expect("failed to read GB file");

    if let Some(last_nonzero) = bytes.iter().rposition(|&b| b != 0) {
        bytes.truncate(last_nonzero + 1);
    } else {
        bytes.clear();
    }

    let byte_list = bytes
        .iter()
        .map(|b| format!("{:#04x}", b))
        .collect::<Vec<_>>()
        .join(", ");
    fs::remove_dir_all(&path).unwrap();

    format!("&[{}]", byte_list).parse().unwrap()
}

#[proc_macro]
pub fn function(_input: TokenStream) -> TokenStream {
    // The generated code is just an expression block returning the function name
    let body = r#"
    {
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }
    "#;

    body.parse().unwrap()
}
