#![allow(unused_must_use)] 
#![allow(non_snake_case)]

extern crate nix;
extern crate bincode;
extern crate rustc_serialize;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use nix::sys::signalfd::*;

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct VirtualMachine {
    IP: usize,
    REG: [u16; 8],
    STACK: Vec<u16>,
    HEAP: Vec<u16>,
    PROGSIZE: usize,
    RUNNING: bool,
}

enum Arg { A = 1, B = 2, C = 3 }

impl VirtualMachine {    

    fn new(bytes: &[u8]) -> VirtualMachine {
        let mut vm = VirtualMachine { IP: 0,
                                      REG: [0; 8],
                                      STACK: Vec::new(),
                                      HEAP: bytes.chunks(2)
                                      .map(|c| (c[0] as u16 | (c[1] as u16) << 8))
                                      .collect(),
                                      PROGSIZE: 0,
                                      RUNNING: true };
        vm.PROGSIZE = vm.HEAP.len();
        vm.HEAP.reserve_exact(0x7fff);
        vm
    }

    fn load(file: String) -> VirtualMachine {
        let mut obj = Vec::new();
        let mut f = File::open(file).unwrap();
        f.read_to_end(&mut obj);
        decode::<VirtualMachine>(&obj[..]).unwrap()
    }
    
    extern fn save(&mut self) {
        self.RUNNING = false;
        let mut f = File::create("vm.save").unwrap();
        f.write_all(&encode(self, SizeLimit::Infinite).unwrap()[..]);
    }
    
    fn run(&mut self) {

        let mut mask = SigSet::empty();
        mask.add(signal::SIGSTOP);
        mask.thread_block().unwrap();
        let mut fd = SignalFd::with_flags(&mask, SFD_NONBLOCK).unwrap();
            
        while self.RUNNING && self.IP < self.PROGSIZE {            
            match self.HEAP[self.IP] {
                0 => self.HALT(),
                1 => self.SET(),
                2 => self.PUSH(),
                3 => self.POP(),
                4 => self.EQ(),
                5 => self.GT(),
                6 => self.JMP(),
                7 => self.JT(),
                8 => self.JF(),
                9 => self.ADD(),
                10 => self.MULT(),
                11 => self.MOD(),
                12 => self.AND(),
                13 => self.OR(),
                14 => self.NOT(),
                15 => self.RMEM(),
                16 => self.WMEM(),
                17 => self.CALL(),
                18 => self.RET(),
                19 => self.OUT(),
                20 => self.IN(),
                21 =>  self.NOOP(),
                _ => ()
            }
            if fd.next().is_some() {
                self.save();
                self.RUNNING = false;
            }
        }
    }
    
    fn _get(&self, i: Arg) -> u16 {
        let v = self.HEAP[self.IP + i as usize];
        if v < 0x8000 {
            return v
        }
        self.REG[0x8000^v as usize]
    }

    fn _set(&mut self, i: Arg, val: u16) {
        let v = self.HEAP[self.IP + i as usize] as usize;
        self.REG[0x8000^v] = val;
    }
    
    fn HALT(&mut self) { self.RUNNING = false }
    fn SET(&mut self) {
        let val = self._get(Arg::B);
        self._set(Arg::A, val);
        self.IP += 3
    }
    fn PUSH(&mut self) {
        let a = self._get(Arg::A);
        self.STACK.push(a);
        self.IP += 2
    }
    fn POP(&mut self) {
        let val = self.STACK.pop().unwrap();
        self._set(Arg::A, val);
        self.IP += 2
    }
    fn EQ(&mut self) {
        let val = (self._get(Arg::B) == self._get(Arg::C)) as u16;
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn GT(&mut self) {
        let val = (self._get(Arg::B) > self._get(Arg::C)) as u16;
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn JMP(&mut self) { self.IP = self._get(Arg::A) as usize }
    fn JT(&mut self) {
        self.IP = if self._get(Arg::A) != 0 { self._get(Arg::B) as usize } else { self.IP + 3 }
    }
    fn JF(&mut self) {
        self.IP = if self._get(Arg::A) == 0 { self._get(Arg::B) as usize } else { self.IP + 3 }
    }
    fn ADD(&mut self) {
        let val = (self._get(Arg::B) + self._get(Arg::C)) % 0x8000;
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn MULT(&mut self) {
        let mut val = 0;
        let mut b = self._get(Arg::B); 
        let mut c = self._get(Arg::C);
        while c > 0 {
            if c % 2 == 1 {
                val = (val + b) % 0x8000;
            }
            b = (b * 2) % 0x8000;
            c /= 2;
        }
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn MOD(&mut self) {
        let val = self._get(Arg::B) % self._get(Arg::C);
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn AND(&mut self) {
        let val = self._get(Arg::B) & self._get(Arg::C);
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn OR(&mut self) {
        let val = self._get(Arg::B) | self._get(Arg::C);
        self._set(Arg::A, val);
        self.IP += 4
    }
    fn NOT(&mut self) {
        let val = 0x7fff^self._get(Arg::B);
        self._set(Arg::A, val);
        self.IP += 3
    }
    fn RMEM(&mut self) {
        let val = self.HEAP[self._get(Arg::B) as usize];
        self._set(Arg::A, val);
        self.IP += 3
    }
    fn WMEM(&mut self) {
        let a = self._get(Arg::A) as usize;
        self.HEAP[a] = self._get(Arg::B);
        self.IP += 3
    }
    fn CALL(&mut self) {
        self.STACK.push(2 + self.IP as u16);
        self.IP = self._get(Arg::A) as usize
    }
    fn RET(&mut self) {
        self.IP = self.STACK.pop().unwrap() as usize
    }
    fn OUT(&mut self) {
        io::stdout().write(&[self._get(Arg::A) as u8]);
        self.IP += 2
    }
    fn IN(&mut self) {
        let mut buf = [0u8];
        io::stdin().read_exact(&mut buf);
        if buf == *b"|" {
            io::stdin().read_exact(&mut buf);
            let mut numbuf: Vec<u8> = Vec::new();
            while buf != *b"|" {
                numbuf.push(buf[0]);
                io::stdin().read_exact(&mut buf);
            }
            self.REG[7] = std::str::from_utf8(&numbuf[..]).unwrap().parse::<u16>().unwrap();
            io::stdin().read_exact(&mut buf);
        }
        self._set(Arg::A, buf[0] as u16);
        self.IP += 2
    }
    fn NOOP(&mut self) {
        self.IP += 1
    }
}

fn main() {
    let mut vm;
    match env::args().nth(2) {
        Some(file) => {
            vm = VirtualMachine::load(file);
        },
        None => {
            vm = VirtualMachine::new(include_bytes!("../../challenge.bin"));
        }
    }
    vm.run()
}
