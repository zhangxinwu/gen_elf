mod config;

use object::{Architecture, BinaryFormat, Endianness};
use object::write::{Object, Symbol, SymbolSection};
use std::fs::File;
use std::io::Write;

use config::*;

fn create_elf_with_function(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个新的 ELF 文件对象
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    obj.flags = object::FileFlags::Elf { os_abi: 2, abi_version: 3, e_flags: 0};


    // 添加 .text section
    let section_id = obj.add_section(vec![], b".text".to_vec(), object::SectionKind::Text);
    let func_body = vec![0xC3]; // `ret` 指令的机器码（用于空实现）
    obj.append_section_data(section_id, &func_body, 1);

    // 定义符号
    let _symbol_id = obj.add_symbol(Symbol {
        name: config.func_name.as_bytes().to_vec(),
        value: 0,
        size: func_body.len() as u64,
        kind: object::SymbolKind::Text,
        scope: object::SymbolScope::Compilation,
        weak: false,
        section: SymbolSection::Section(section_id),
        flags: object::SymbolFlags::None,
    });

    // 将对象写入文件
    let mut file = File::create("output.o")?;
    file.write_all(&obj.write()?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config("config.yaml")?;
    create_elf_with_function(&config)?;
    println!("ELF 文件生成成功: output.o");
    Ok(())
}
