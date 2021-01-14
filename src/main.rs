use std::env;
use std::fs::File;
use std::io::prelude::*;
mod cpu;

fn read_rom() -> Vec<u8> {
    let file_name = env::args().nth(1).unwrap();
    let mut file = File::open(file_name).unwrap();
    let mut rom_buf = Vec::new();
    file.read_to_end(&mut rom_buf).unwrap();

    rom_buf
}

fn main() {
    let rom_buf = read_rom();
    let cpu = cpu::CPU::new(&rom_buf);
    println!("{:?}", cpu.opcodes);
    cpu.opcodes
        .iter()
        .for_each(|opcode| cpu.run_instruction(*opcode));
}
