#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::mem;

pub struct Codegen {
    writer: Box<dyn io::Write>,
}

impl Codegen {
    pub fn new(writer: impl io::Write + 'static) -> Self {
        Self {
            writer: Box::new(writer),
        }
    }

    pub fn create_file(path: &str) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self::new(BufWriter::with_capacity(1024 * 1024, file)))
    }
}

impl Codegen {
    pub fn lf(&mut self) {
        writeln!(self.writer).unwrap();
    }

    pub fn ln(&mut self, line: impl AsRef<str>) {
        writeln!(self.writer, "{}", line.as_ref()).unwrap();
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Codegen>> = RefCell::new(None);
}

pub fn scoped<T>(g: Codegen, f: impl FnOnce()) -> Codegen {
    let prev = CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        cur.replace(g)
    });

    f();

    CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        mem::replace(&mut *cur, prev).unwrap()
    })
}

pub fn with(f: impl FnOnce(&mut Codegen)) {
    CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        let g = cur.as_mut().expect("codegen is not in scope");
        f(g);
    })
}

#[macro_export]
macro_rules! g {
    [$($line:expr,)+] => {
        $crate::with(|g| {
            $(
                g.ln($line);
            )+
        })
    };
    () => {
        $crate::with(|g| g.lf())
    };
    ($fmt: literal) => {
        $crate::with(|g| g.ln($fmt))
    };
    ($fmt: literal, $($arg: tt)*) => {
        $crate::with(|g| g.ln(format!($fmt, $($arg)*)))
    };
}
