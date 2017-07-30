use std::collections::HashMap;

use instruction::*;

type Address = u16;

pub struct Executable {
    raw: Vec<u16>,
    imap: HashMap<Address, Instruction>,
}

macro_rules! insert {
    ($obj:ident $t:ident $i:ident+$incrm:expr) => {
        {
            $obj.imap.insert($i as u16, Instruction::new(InstructionSet::$t));
            println!("Inserting {:?} at address {:x}", InstructionSet::$t, $i); 
            $i += $incrm;
        }
    }
}

impl Executable {

    pub fn new(bytes: &[u8]) -> Executable {
        let mut exe = Executable {
            raw: bytes.chunks(2)                                    
                .map(|c| (c[0] as u16 | (c[1] as u16) << 8))
                .collect(),
            imap: HashMap::new()
        };

        let mut pos: usize = 0;
        while pos < exe.raw.len() {
            match exe.raw[pos] {
                0 => insert![exe HALT pos+1],
                1 => insert![exe SET pos+3],
                2 => insert![exe PUSH pos+2],
                3 => insert![exe POP pos+2],
                4 => insert![exe EQ pos+4],
                5 => insert![exe GT pos+4],
	        6 => insert![exe JMP pos+2],
                7 => insert![exe JT pos+3],
                8 => insert![exe JF pos+3],
                9 => insert![exe ADD pos+4],
                10 => insert![exe MULT pos+4],
                11 => insert![exe MOD pos+4],
                12 => insert![exe AND pos+4],
	        13 => insert![exe OR pos+4],
                14 => insert![exe NOT pos+3],
                15 => insert![exe RMEM pos+3],
                16 => insert![exe WMEM pos+3],
                17 => insert![exe CALL pos+2],
                18 => insert![exe RET pos+1],
                19 => insert![exe OUT pos+2],
                20 => insert![exe IN pos+2],
                21 => insert![exe NOOP pos+1],
                _ => { pos += 1; }
            }
        }
        return exe
    }

    fn disass_proc (&mut self, start_address: u16) -> bool {
        let mut i = start_address;
        loop {
            if let Some(instr) = self.imap.get_mut(&i) {
                println!("{:?}", instr.instr);
                i += instr.size;
                if instr.instr == InstructionSet::RET {
                    break;
                }
            }
            else {
                println!("Address not found!");
                return false
            }
        }
        true
    }
    
    pub fn disassemble(&mut self) {

        let call_addr = self.imap.iter().filter_map(|(a, i)| {
            if i.instr == InstructionSet::CALL {
                Some(*a)
            }
            else {
                None
            }
        }).collect::<Vec<u16>>();
        
        for addr in call_addr  {
            let proc_addr = self.raw[addr as usize + 1];
            if proc_addr < 0x8000 {
                println!("Procedure at address {:x}?", addr);
                let success = self.disass_proc(proc_addr);
                if success {
                    self.imap.get_mut(&addr).unwrap().typ = Type::StaticCall;
                }
            }
            else {
                self.imap.get_mut(&addr).unwrap().typ = Type::DynamicCall;
            }
        }
    }
}
