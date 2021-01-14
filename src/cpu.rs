#[derive(Debug)]
pub enum OpCodes {
    _0NNN(u16),
    _00E0(u16),
    _00EE(u16),
    _1NNN(u16),
    _2NNN(u16),
    _3XNN(u16),
}

#[derive(Debug)]
pub struct CPU {
    pub v_reg: [u8; 16],
    pub i_reg: u16,
    pub delay_reg: u8,
    pub sound_reg: u8,
    pub prog_counter: u16,
    pub stack_ptr: u8,
    pub stack: [u16; 16],
    pub opcodes: Vec<String>,
}

impl CPU {
    pub fn new(rom_buf: &[u8]) -> Self {
        let opcodes = CPU::convert_rom_to_opcodes(rom_buf);
        CPU {
            v_reg: [0; 16],
            i_reg: 0,
            delay_reg: 0,
            sound_reg: 0,
            prog_counter: 0,
            stack_ptr: 0,
            stack: [0; 16],
            opcodes,
        }
    }
    // pub fn run_instruction(&self, opcode: &str) {
    //     // Convert opcode to
    //     match opc {
    //         _0NNN => println!("0NNN"),
    //         _00E0 => println!("00E0"),
    //         _00EE => println!("00EE"),
    //         _1NNN => println!("1NNN"),
    //         _2NNN => println!("2NNN"),
    //         _3XNN => println!("3XNN"),
    //     }
    // }

    pub fn convert_rom_to_opcodes(rom_buf: &[u8]) -> Vec<String> {
        let mut opcodes: Vec<String> = Vec::new();
        for index in 0..(rom_buf.len() / 2) {
            let val0 = rom_buf[2 * index];
            let val1 = rom_buf[2 * index + 1];
            let opcode = ((val0 as u16) << 8) | val1 as u16;
            opcodes.push(format!("{:04X}", opcode));
        }
        opcodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    // This is probably not the right way to do this, oh well...
    fn read_test_opcode() -> Vec<u8> {
        let mut file = File::open("roms/ibm_logo.ch8").unwrap();
        let mut rom_buf = Vec::new();
        file.read_to_end(&mut rom_buf).unwrap();
        rom_buf
    }
    #[test]
    fn opcode_conversion_successful() {
        let test_opcode = read_test_opcode();
        let cpu = CPU::new(&test_opcode);
        // Just test the first few to make sure they're correct
        assert_eq!(
            cpu.opcodes[0..=5],
            vec!["00E0", "A22A", "600C", "6108", "D01F", "7009",]
        );
        // And test the last two to make sure it works
        assert_eq!(cpu.opcodes[(cpu.opcodes.len() - 2)..], vec!["00E0", "00E0"]);
    }
}

// pub mod Instructions {
//     // 0NNN
//     pub fn exec_subroutine(opcode: u16) {}
//     // 00E0
//     pub fn clear_screen(opcode: u16) {}
//     // 00EE
//     pub fn return_from_subroutine(opcode: u16) {}
//     // 1NNN
//     pub fn jump_to_address(opcode: u16) {}
//     // 2NNN
//     pub fn exec_subroutine_from_address(opcode: u16) {}
//     // 3XNN
//     pub fn skip_if_vx_equals_value(opcode: u16) {}
//     // 4XNN
//     pub fn skip_if_vx_not_equals_value(opcode: u16) {}
//     // 5XY0
//     pub fn skip_if_vx_equals_vy(opcode: u16) {}
//     // 6XNN
//     pub fn store_value_in_vx(opcode: u16) {}
//     // 7XNN
//     pub fn add_value_to_vx(opcode: u16) {}
//     // 8XY0
//     pub fn store_vy_in_vx(opcode: u16) {}
//     // 8XY1
//     pub fn set_vx_to_vx_or_vy(opcode: u16) {}
//     // 8XY2
//     pub fn set_vx_to_vx_and_vy(opcode: u16) {}
//     // 8XY3
//     pub fn set_vx_to_vx_xor_vy(opcode: u16) {}
//     // 8XY4
//     pub fn add_vy_to_vx(opcode: u16) {}
//     // 8XY5
//     pub fn subtract_vy_to_vx(opcode: u16) {}
//     // 8XY6
//     pub fn store_vy_shifted_right_in_vx(opcode: u16) {}
//     // 8XY7
//     pub fn set_vx_to_vy_minus_vx(opcode: u16) {}
//     // 8XYE
//     pub fn store_vy_shifted_left_in_vx(opcode: u16) {}
//     // 9XY0
//     pub fn skip_if_vx_not_equals_vy(opcode: u16) {}
//     // ANNN
//     pub fn store_value_in_reg_i(opcode: u16) {}
//     // BNNN
//     pub fn jump_to_address_value_plus_v0(opcode: u16) {}
//     // CXNN
//     pub fn set_vx_to_random_number_with_mask_of_nn(opcode: u16) {}
//     // DXYN
//     pub fn draw_sprite_at_position_vx_vy(opcode: u16) {}
//     // EX9E
//     pub fn skip_instruction_if_key_pressed_matches_vx(opcode: u16) {}
//     // EXA1
//     pub fn skip_instruction_if_not_key_pressed_matches_vx(opcode: u16) {}
//     // FX07
//     pub fn store_delay_timer_value_in_vx(opcode: u16) {}
//     // FX0A
//     pub fn wait_for_keypress_and_store_in_vx(opcode: u16) {}
//     // FX15
//     pub fn set_delay_timer_to_vx(opcode: u16) {}
//     // FX18
//     pub fn set_sound_timer_to_vx(opcode: u16) {}
//     // FX1E
//     pub fn add_vx_to_reg_i(opcode: u16) {}
//     // FX29
//     pub fn set_reg_i_to_address_of_sprite_data_from_vx(opcode: u16) {}
//     // FX33
//     pub fn store_bcd_of_vx_at_reg_i_1_2_3_LOL(opcode: u16) {}
//     // FX55
//     pub fn store_v0_to_vx_in_memory_address_from_reg_i(opcode: u16) {}
//     // FX65
//     pub fn fill_v0_to_vx_from_memory_address_from_reg_i(opcode: u16) {}
// }
