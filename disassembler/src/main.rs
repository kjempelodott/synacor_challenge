mod instruction;
mod executable;

use self::executable::Executable;


fn main() {
    let mut exc = Executable::new(include_bytes!("../../challenge.bin"));
    exc.disassemble();
}
