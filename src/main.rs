mod config;

use config::*;
use object::write::Object as WObject;
use object::write::Symbol as WSymbol;
use object::write::{SymbolFlags as WSymbolFlags, SymbolSection as WSymbolSection};
use object::Architecture::X86_64;
use object::BinaryFormat::Elf;
use object::{Object as RObject, ObjectSegment, Segment};
use object::{
    Architecture, BinaryFormat, Endianness, ObjectSection, ObjectSymbol,
    SymbolFlags as RSymbolFlags,
};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Not;

fn create_elf_with_function(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个新的 ELF 文件对象
    let mut obj = WObject::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    obj.flags = object::FileFlags::Elf {
        os_abi: 2,
        abi_version: 3,
        e_flags: 0,
    };

    // 添加 .text section
    let section_id = obj.add_section(vec![], b".text".to_vec(), object::SectionKind::Text);
    let func_body = vec![0xC3]; // `ret` 指令的机器码（用于空实现）
    obj.append_section_data(section_id, &func_body, 1);

    // 定义符号
    let _symbol_id = obj.add_symbol(WSymbol {
        name: config.func_name.as_bytes().to_vec(),
        value: 0,
        size: func_body.len() as u64,
        kind: object::SymbolKind::Text,
        scope: object::SymbolScope::Compilation,
        weak: false,
        section: WSymbolSection::Section(section_id),
        flags: object::SymbolFlags::None,
    });

    // 将对象写入文件
    let mut file = File::create("output.o")?;
    file.write_all(&obj.write()?)?;
    Ok(())
}

fn pa() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("libart.so")?;
    let so = object::File::parse(&*data)?;
    let mut f = WObject::new(Elf, X86_64, Endianness::Little);
    f.flags = so.flags();
    let mut hash_map = HashMap::new();
    for segment in so.segments() {
        println!("segment: {:?} {:?}", segment.name(), segment);
    }
    println!("-0--------------");
    for section in so.sections() {
        let section_id = f.add_section(
            {
                if let Ok(Some(name)) = section.segment_name_bytes() {
                    name.to_vec()
                } else {
                    vec![]
                }
            },
            Vec::from(section.name()?),
            section.kind(),
        );
        hash_map.insert(section.index(), section_id);
        println!("add_section {} {:?} {:?}", section.name()?, section.index(), section_id);
        if section.name()?.eq(".bss").not() {
            println!("append_section_data data_size {:?} align {:?}", section.size(), section.align());
            f.set_section_data(section_id, section.data()?, section.align());
        }
    }
    for symbol in so.symbols() {
        if symbol.section_index().is_some() {
            // println!("add_symbol {} {:?}", symbol.name()?, symbol.section_index());
            f.add_symbol(WSymbol {
                name: Vec::from(symbol.name()?),
                value: symbol.address(),
                size: symbol.size(),
                kind: symbol.kind(),
                scope: symbol.scope(),
                weak: symbol.is_weak(),
                section: WSymbolSection::Section(
                    hash_map
                        .get(&symbol.section_index().unwrap())
                        .copied()
                        .unwrap(),
                ),
                flags: {
                    if let RSymbolFlags::Elf { st_info, st_other } = symbol.flags() {
                        WSymbolFlags::Elf { st_info, st_other }
                    } else {
                        WSymbolFlags::None
                    }
                },
            });
        }
    }
    let mut file = File::create("fake_libart.so")?;
    file.write_all(&f.write()?)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config("config.yaml")?;
    create_elf_with_function(&config)?;
    println!("ELF 文件生成成功: output.o");
    pa().expect("TODO: panic message");
    Ok(())
}
