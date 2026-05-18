use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmObjectInspection {
    pub object_format: String,
    pub wasm_magic_valid: bool,
    pub wasm_version_valid: bool,
    pub object_section_count: u64,
    pub object_sections: Vec<WasmObjectSection>,
    pub object_has_code_section: bool,
    pub object_has_linking_metadata: bool,
    pub object_has_relocation_sections: bool,
    pub object_symbol_count: u64,
    pub object_symbols: Vec<WasmObjectSymbol>,
    pub object_function_count: u64,
    pub object_imported_function_count: u64,
    pub object_export_count: u64,
    pub object_imports: Vec<String>,
    pub object_exports: Vec<String>,
    pub object_has_code_bearing_content: bool,
    pub object_is_empty: bool,
    pub parse_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmObjectSection {
    pub index: u64,
    pub id: u8,
    pub name: String,
    pub custom_name: Option<String>,
    pub offset: u64,
    pub payload_offset: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmObjectSymbol {
    pub index: u64,
    pub kind: String,
    pub flags: u32,
    pub wasm_index: Option<u32>,
    pub name: Option<String>,
    pub undefined: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    limit: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], offset: usize, limit: usize) -> Self {
        Self {
            bytes,
            offset,
            limit,
        }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn is_done(&self) -> bool {
        self.offset >= self.limit
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let Some(byte) = self.bytes.get(self.offset).copied() else {
            return Err("unexpected end of section while reading byte".to_owned());
        };
        if self.offset >= self.limit {
            return Err("section read crossed declared limit".to_owned());
        }
        self.offset += 1;
        Ok(byte)
    }

    fn read_varuint(&mut self) -> Result<u32, String> {
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u32)
                .checked_shl(shift)
                .ok_or_else(|| "varuint shift overflow".to_owned())?;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 28 {
                return Err("varuint is too large".to_owned());
            }
        }
    }

    fn read_varuint64(&mut self) -> Result<u64, String> {
        let mut result = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7f) as u64)
                .checked_shl(shift)
                .ok_or_else(|| "varuint64 shift overflow".to_owned())?;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 63 {
                return Err("varuint64 is too large".to_owned());
            }
        }
    }

    fn read_name(&mut self) -> Result<String, String> {
        let len = self.read_varuint()? as usize;
        let start = self.offset;
        let end = start
            .checked_add(len)
            .ok_or_else(|| "name length overflow".to_owned())?;
        if end > self.limit || end > self.bytes.len() {
            return Err("name extends past section limit".to_owned());
        }
        self.offset = end;
        std::str::from_utf8(&self.bytes[start..end])
            .map(str::to_owned)
            .map_err(|error| format!("invalid UTF-8 name: {error}"))
    }
}

#[derive(Debug, Default)]
struct Accumulator {
    imported_function_count: u64,
    defined_function_count: u64,
    code_body_count: u64,
    export_count: u64,
    symbol_count: u64,
    symbols: Vec<WasmObjectSymbol>,
    imports: Vec<String>,
    exports: Vec<String>,
    has_code_section: bool,
    has_linking_metadata: bool,
    has_relocation_sections: bool,
    errors: Vec<String>,
}

pub fn inspect_wasm_object(bytes: &[u8]) -> WasmObjectInspection {
    let wasm_magic_valid = bytes.len() >= 4 && &bytes[..4] == b"\0asm";
    let wasm_version_valid = bytes.len() >= 8 && &bytes[4..8] == b"\x01\0\0\0";
    let mut acc = Accumulator::default();
    let mut sections = Vec::new();

    if !wasm_magic_valid || !wasm_version_valid {
        let reason = if !wasm_magic_valid {
            "invalid WebAssembly magic"
        } else {
            "invalid WebAssembly version"
        };
        acc.errors.push(reason.to_owned());
        return finish_inspection(wasm_magic_valid, wasm_version_valid, sections, acc);
    }

    let mut reader = Reader::new(bytes, 8, bytes.len());
    let mut section_index = 0u64;
    while !reader.is_done() {
        let section_start = reader.offset();
        let section_id = match reader.read_u8() {
            Ok(section_id) => section_id,
            Err(error) => {
                acc.errors.push(error);
                break;
            }
        };
        let section_size = match reader.read_varuint() {
            Ok(size) => size as usize,
            Err(error) => {
                acc.errors.push(format!(
                    "section {section_index} length decode failed: {error}"
                ));
                break;
            }
        };
        let payload_offset = reader.offset();
        let Some(section_end) = payload_offset.checked_add(section_size) else {
            acc.errors
                .push(format!("section {section_index} length overflow"));
            break;
        };
        if section_end > bytes.len() {
            acc.errors.push(format!(
                "section {section_index} extends past end of module"
            ));
            break;
        }

        let custom_name = if section_id == 0 {
            parse_custom_section(bytes, payload_offset, section_end, &mut acc)
        } else {
            parse_known_section(section_id, bytes, payload_offset, section_end, &mut acc);
            None
        };

        sections.push(WasmObjectSection {
            index: section_index,
            id: section_id,
            name: section_name(section_id).to_owned(),
            custom_name,
            offset: section_start as u64,
            payload_offset: payload_offset as u64,
            size_bytes: section_size as u64,
        });
        reader.offset = section_end;
        section_index += 1;
    }

    finish_inspection(wasm_magic_valid, wasm_version_valid, sections, acc)
}

fn finish_inspection(
    wasm_magic_valid: bool,
    wasm_version_valid: bool,
    sections: Vec<WasmObjectSection>,
    acc: Accumulator,
) -> WasmObjectInspection {
    let object_has_code_bearing_content = acc.has_code_section && acc.code_body_count > 0;
    WasmObjectInspection {
        object_format: if wasm_magic_valid && wasm_version_valid {
            "wasm_object".to_owned()
        } else {
            "invalid_wasm_object".to_owned()
        },
        wasm_magic_valid,
        wasm_version_valid,
        object_section_count: sections.len() as u64,
        object_sections: sections,
        object_has_code_section: acc.has_code_section,
        object_has_linking_metadata: acc.has_linking_metadata,
        object_has_relocation_sections: acc.has_relocation_sections,
        object_symbol_count: acc.symbol_count,
        object_symbols: acc.symbols,
        object_function_count: acc.code_body_count.max(acc.defined_function_count),
        object_imported_function_count: acc.imported_function_count,
        object_export_count: acc.export_count,
        object_imports: acc.imports,
        object_exports: acc.exports,
        object_has_code_bearing_content,
        object_is_empty: !object_has_code_bearing_content,
        parse_errors: acc.errors,
    }
}

fn parse_known_section(
    section_id: u8,
    bytes: &[u8],
    payload_offset: usize,
    section_end: usize,
    acc: &mut Accumulator,
) {
    let mut reader = Reader::new(bytes, payload_offset, section_end);
    match section_id {
        2 => {
            if let Err(error) = parse_import_section(&mut reader, acc) {
                acc.errors
                    .push(format!("import section parse failed: {error}"));
            }
        }
        3 => match reader.read_varuint() {
            Ok(count) => acc.defined_function_count = count as u64,
            Err(error) => acc
                .errors
                .push(format!("function section parse failed: {error}")),
        },
        7 => {
            if let Err(error) = parse_export_section(&mut reader, acc) {
                acc.errors
                    .push(format!("export section parse failed: {error}"));
            }
        }
        10 => {
            acc.has_code_section = true;
            match reader.read_varuint() {
                Ok(count) => acc.code_body_count = count as u64,
                Err(error) => acc
                    .errors
                    .push(format!("code section parse failed: {error}")),
            }
        }
        _ => {}
    }
}

fn parse_custom_section(
    bytes: &[u8],
    payload_offset: usize,
    section_end: usize,
    acc: &mut Accumulator,
) -> Option<String> {
    let mut reader = Reader::new(bytes, payload_offset, section_end);
    match reader.read_name() {
        Ok(name) => {
            if name == "linking" {
                acc.has_linking_metadata = true;
                if let Err(error) = parse_linking_section(&mut reader, acc) {
                    acc.errors
                        .push(format!("linking section parse failed: {error}"));
                }
            } else if name.starts_with("reloc.") {
                acc.has_relocation_sections = true;
            }
            Some(name)
        }
        Err(error) => {
            acc.errors
                .push(format!("custom section name parse failed: {error}"));
            None
        }
    }
}

fn parse_import_section(reader: &mut Reader<'_>, acc: &mut Accumulator) -> Result<(), String> {
    let count = reader.read_varuint()?;
    for _ in 0..count {
        let module = reader.read_name()?;
        let name = reader.read_name()?;
        let kind = reader.read_u8()?;
        acc.imports.push(format!("{module}::{name}"));
        match kind {
            0 => {
                acc.imported_function_count += 1;
                let _type_index = reader.read_varuint()?;
            }
            1 => skip_table_type(reader)?,
            2 => skip_memory_type(reader)?,
            3 => skip_global_type(reader)?,
            _ => return Err(format!("unsupported import kind {kind}")),
        }
    }
    Ok(())
}

fn parse_export_section(reader: &mut Reader<'_>, acc: &mut Accumulator) -> Result<(), String> {
    let count = reader.read_varuint()?;
    acc.export_count = count as u64;
    for _ in 0..count {
        let name = reader.read_name()?;
        let _kind = reader.read_u8()?;
        let _index = reader.read_varuint()?;
        acc.exports.push(name);
    }
    Ok(())
}

fn parse_linking_section(reader: &mut Reader<'_>, acc: &mut Accumulator) -> Result<(), String> {
    if reader.is_done() {
        return Ok(());
    }
    let _version = reader.read_varuint()?;
    while !reader.is_done() {
        let subsection_id = reader.read_u8()?;
        let subsection_size = reader.read_varuint()? as usize;
        let start = reader.offset();
        let end = start
            .checked_add(subsection_size)
            .ok_or_else(|| "linking subsection length overflow".to_owned())?;
        if end > reader.limit || end > reader.bytes.len() {
            return Err("linking subsection extends past custom section".to_owned());
        }
        if subsection_id == 8 {
            let mut symbol_reader = Reader::new(reader.bytes, start, end);
            parse_symbol_table(&mut symbol_reader, acc)?;
        }
        reader.offset = end;
    }
    Ok(())
}

const WASM_SYMBOL_UNDEFINED: u32 = 0x10;
const WASM_SYMBOL_EXPLICIT_NAME: u32 = 0x40;
const WASM_SYMBOL_ABSOLUTE: u32 = 0x200;

fn parse_symbol_table(reader: &mut Reader<'_>, acc: &mut Accumulator) -> Result<(), String> {
    let count = reader.read_varuint()?;
    acc.symbol_count = count as u64;
    for index in 0..count {
        let kind = reader.read_u8()?;
        let flags = reader.read_varuint()?;
        let undefined = flags & WASM_SYMBOL_UNDEFINED != 0;
        let (wasm_index, name) = match kind {
            0 | 2 | 4 | 5 => {
                let element_index = reader.read_varuint()?;
                let name = if !undefined || flags & WASM_SYMBOL_EXPLICIT_NAME != 0 {
                    Some(reader.read_name()?)
                } else {
                    None
                };
                (Some(element_index), name)
            }
            1 => {
                let name = Some(reader.read_name()?);
                if !undefined {
                    let segment_index = reader.read_varuint()?;
                    if flags & WASM_SYMBOL_ABSOLUTE == 0 {
                        let _offset = reader.read_varuint64()?;
                        let _size = reader.read_varuint64()?;
                    } else {
                        let _absolute_offset = reader.read_varuint64()?;
                        let _size = reader.read_varuint64()?;
                    }
                    (Some(segment_index), name)
                } else {
                    (None, name)
                }
            }
            3 => {
                let section_index = reader.read_varuint()?;
                (Some(section_index), None)
            }
            _ => return Err(format!("unsupported wasm symbol kind {kind}")),
        };
        acc.symbols.push(WasmObjectSymbol {
            index: index as u64,
            kind: symbol_kind_name(kind).to_owned(),
            flags,
            wasm_index,
            name,
            undefined,
        });
    }
    Ok(())
}

fn skip_limits(reader: &mut Reader<'_>) -> Result<(), String> {
    let flags = reader.read_u8()?;
    let _min = reader.read_varuint()?;
    if flags & 0x01 != 0 {
        let _max = reader.read_varuint()?;
    }
    Ok(())
}

fn skip_table_type(reader: &mut Reader<'_>) -> Result<(), String> {
    let _element_type = reader.read_u8()?;
    skip_limits(reader)
}

fn skip_memory_type(reader: &mut Reader<'_>) -> Result<(), String> {
    skip_limits(reader)
}

fn skip_global_type(reader: &mut Reader<'_>) -> Result<(), String> {
    let _value_type = reader.read_u8()?;
    let _mutability = reader.read_u8()?;
    Ok(())
}

fn section_name(section_id: u8) -> &'static str {
    match section_id {
        0 => "custom",
        1 => "type",
        2 => "import",
        3 => "function",
        4 => "table",
        5 => "memory",
        6 => "global",
        7 => "export",
        8 => "start",
        9 => "element",
        10 => "code",
        11 => "data",
        12 => "data_count",
        _ => "unknown",
    }
}

fn symbol_kind_name(kind: u8) -> &'static str {
    match kind {
        0 => "function",
        1 => "data",
        2 => "global",
        3 => "section",
        4 => "tag",
        5 => "table",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varuint(mut value: u32) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                return out;
            }
        }
    }

    fn named_custom_section(name: &str, payload: &[u8]) -> Vec<u8> {
        let mut section = Vec::new();
        section.extend(varuint(name.len() as u32));
        section.extend(name.as_bytes());
        section.extend(payload);
        let mut out = vec![0];
        out.extend(varuint(section.len() as u32));
        out.extend(section);
        out
    }

    fn linking_symbol_table(symbols: Vec<Vec<u8>>) -> Vec<u8> {
        let mut symbol_table = Vec::new();
        symbol_table.extend(varuint(symbols.len() as u32));
        for mut symbol in symbols {
            symbol_table.append(&mut symbol);
        }

        let mut payload = Vec::new();
        payload.extend(varuint(2));
        payload.push(8);
        payload.extend(varuint(symbol_table.len() as u32));
        payload.extend(symbol_table);
        payload
    }

    fn function_symbol(index: u32, name: &str) -> Vec<u8> {
        let mut symbol = vec![0];
        symbol.extend(varuint(0));
        symbol.extend(varuint(index));
        symbol.extend(varuint(name.len() as u32));
        symbol.extend(name.as_bytes());
        symbol
    }

    #[test]
    fn detects_empty_wasm_object_with_linking_metadata() {
        let mut module = b"\0asm\x01\0\0\0".to_vec();
        module.extend(named_custom_section("linking", &[2, 8, 1, 0]));

        let inspection = inspect_wasm_object(&module);

        assert_eq!(inspection.object_format, "wasm_object");
        assert!(inspection.wasm_magic_valid);
        assert!(inspection.wasm_version_valid);
        assert_eq!(inspection.object_section_count, 1);
        assert!(inspection.object_has_linking_metadata);
        assert_eq!(inspection.object_symbol_count, 0);
        assert!(inspection.object_symbols.is_empty());
        assert_eq!(inspection.object_function_count, 0);
        assert!(!inspection.object_has_code_section);
        assert!(!inspection.object_has_code_bearing_content);
        assert!(inspection.object_is_empty);
    }

    #[test]
    fn counts_code_bearing_defined_functions() {
        let mut type_section = vec![1, 0x60, 0, 0];
        let mut function_section = vec![1, 0];
        let mut body = vec![2, 0, 0x0b];
        let mut code_section = vec![1];
        code_section.append(&mut body);

        let mut module = b"\0asm\x01\0\0\0".to_vec();
        module.push(1);
        module.extend(varuint(type_section.len() as u32));
        module.append(&mut type_section);
        module.push(3);
        module.extend(varuint(function_section.len() as u32));
        module.append(&mut function_section);
        module.push(10);
        module.extend(varuint(code_section.len() as u32));
        module.append(&mut code_section);

        let inspection = inspect_wasm_object(&module);

        assert_eq!(inspection.object_function_count, 1);
        assert!(inspection.object_has_code_section);
        assert!(inspection.object_has_code_bearing_content);
        assert!(!inspection.object_is_empty);
        assert!(inspection.parse_errors.is_empty());
    }

    #[test]
    fn parses_linking_symbol_table_function_names() {
        let mut type_section = vec![1, 0x60, 0, 0];
        let mut function_section = vec![1, 0];
        let mut body = vec![2, 0, 0x0b];
        let mut code_section = vec![1];
        code_section.append(&mut body);

        let mut module = b"\0asm\x01\0\0\0".to_vec();
        module.push(1);
        module.extend(varuint(type_section.len() as u32));
        module.append(&mut type_section);
        module.push(3);
        module.extend(varuint(function_section.len() as u32));
        module.append(&mut function_section);
        module.push(10);
        module.extend(varuint(code_section.len() as u32));
        module.append(&mut code_section);
        module.extend(named_custom_section(
            "linking",
            &linking_symbol_table(vec![function_symbol(0, "_RNvC8rouwdi_4main")]),
        ));

        let inspection = inspect_wasm_object(&module);

        assert_eq!(inspection.object_symbol_count, 1);
        assert_eq!(inspection.object_symbols.len(), 1);
        assert_eq!(inspection.object_symbols[0].kind, "function");
        assert_eq!(
            inspection.object_symbols[0].name.as_deref(),
            Some("_RNvC8rouwdi_4main")
        );
        assert!(!inspection.object_symbols[0].undefined);
        assert_eq!(inspection.object_function_count, 1);
        assert!(inspection.object_has_code_bearing_content);
        assert!(inspection.parse_errors.is_empty());
    }
}
