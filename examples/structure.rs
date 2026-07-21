use cpp_demangle::{DemangleNodeType, DemangleOptions, DemangleWrite, Symbol};
use std::fmt;

struct Dump {
    depth: usize,
}

impl DemangleWrite for Dump {
    fn push_demangle_node(&mut self, t: DemangleNodeType) {
        println!("{}+ {:?}", "  ".repeat(self.depth), t);
        self.depth += 1;
    }
    fn write_string(&mut self, s: &str) -> fmt::Result {
        println!("{}  {:?}", "  ".repeat(self.depth), s);
        Ok(())
    }
    fn pop_demangle_node(&mut self) {
        self.depth = self.depth.saturating_sub(1);
        println!("{}-", "  ".repeat(self.depth));
    }
}

fn main() {
    for sym in std::env::args().skip(1) {
        println!("=== {sym} ===");
        match Symbol::new(sym.into_bytes()) {
            Ok(s) => {
                let mut d = Dump { depth: 0 };
                let _ = s.structured_demangle(&mut d, &DemangleOptions::new());
            }
            Err(e) => println!("parse err: {e:?}"),
        }
    }
}
