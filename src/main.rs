include!(concat!(env!("OUT_DIR"), "/asm_blobs.rs"));
use pymem::Pymem;
use std::io::{self, BufRead, Write};

#[repr(C)]
pub struct shared_memory {
    pub enabled: bool,
    pub old: usize,
    pub luau_compile: usize,
    pub luau_load: usize,
    pub defer: usize,
    pub lua_gettop: usize,
    pub lua_settop: usize,
    pub script: usize,
    pub script_len: usize,
}

mod offsets {
    pub static OLD: usize = 0x1F5F10;
    pub static LUAU_COMPILE: usize = 0x9443F0;
    pub static LUAU_LOAD: usize = 0x9395B0;
    pub static DEFER: usize = 0x1ECE40;
    pub static LUA_GETTOP: usize = 0x91A380;
    pub static LUA_SETTOP: usize = 0x91A390;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pm: Pymem = Pymem::open_process_from_name("LuduvoEditor.exe").expect("Luduvo must be running");

    let base: usize = pm.base_address()?;
    println!("base: {:#X}", base);

    let sm: shared_memory = shared_memory {
        enabled: false,
        old: (base + offsets::OLD) - 0x67,
        luau_compile: base + offsets::LUAU_COMPILE,
        luau_load: base + offsets::LUAU_LOAD,
        defer: base + offsets::DEFER,
        lua_gettop: base + offsets::LUA_GETTOP,
        lua_settop: base + offsets::LUA_SETTOP,
        script: 0,
        script_len: 0,
    };

    let bytes: &[u8] = unsafe{ std::slice::from_raw_parts(&sm as *const shared_memory as *const u8, std::mem::size_of::<shared_memory>()) };
    let sm_addr: usize = pm.allocate(bytes.len())?;
    pm.write_bytes(sm_addr, bytes)?;
    println!("shared_memory: {:#X}", sm_addr);

    let mut asm: Vec<u8> = ASM_EXECUTOR.to_vec();
    if let Some(pos) = asm.windows(8).position(|w: &[u8]| w == 0x1151721933u64.to_le_bytes()) {
        asm[pos..pos + 8].copy_from_slice(&(sm_addr as u64).to_le_bytes());
    }

    let addr: usize = pm.allocate(asm.len())?;
    pm.write_bytes(addr, &asm)?;
    println!("asm: {:#X}", addr);

    let (mut scan, mut count) = (0, 0);
    while let Ok(region) = pm.virtual_query(scan) {
        if region.protect & 0x04 != 0 && region.protect & 0x10 == 0 && region.region_type & 0x20000 != 0 {
            if let Ok(buf) = pm.read_bytes(region.base_address, region.region_size) {
                for (i, w) in buf.windows(8).enumerate() {
                    if w == ((base + offsets::OLD) as u64).to_le_bytes() {
                        pm.write_ptr(region.base_address + i, addr)?;
                        count += 1;
                    }
                }
            }
        }
        scan = region.base_address + region.region_size;
    }

    println!("replaced {} func(s)\n----------------------------\n", count);

    let mut script: usize = 0;
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut line: String = String::new();

        if io::stdin().lock().read_line(&mut line)? == 0 {
            break;
        }

        let line: &str = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts: std::str::SplitN<'_, char> = line.splitn(2, ' ');
        let cmd: &str = parts.next().unwrap_or("");
        let arg: &str = parts.next().unwrap_or("").trim();

        match cmd {
            "run" => {
                if !arg.is_empty() {
                    let new: &[u8] = arg.as_bytes();
                    let new_addr: usize = pm.allocate(new.len())?;
                    pm.write_bytes(new_addr, new)?;

                    let old_script: usize = script;
                    script = new_addr;

                    pm.write_ptr(sm_addr + 0x38, new_addr)?;
                    pm.write_bytes(sm_addr + 0x40, &(new.len() as u64).to_le_bytes())?;

                    if old_script != 0 {
                        let _ = pm.free(old_script);
                    }
                }
                pm.write_bytes(sm_addr, &[1u8])?;
            }
            "exit" => break,
            _ => println!("cmds: run [script], exit"),
        }
    }

    return Ok(());
}