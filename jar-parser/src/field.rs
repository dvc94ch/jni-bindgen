//! [Java SE 7 &sect; 4.5](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.5):  Parsing APIs and structures for class fields.

use super::*;
use crate::io::*;

use bitflags::bitflags;

use std::io::{self, Read};



bitflags! {
    #[derive(Default)]
    /// [Java SE 7 &sect; 4.5](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.5):  field_info::access_flags
    pub struct Flags : u16 {
        /// Declared `public`; may be accessed from outside its package.
        const PUBLIC        = 0x0001;
        /// Declared `private`; usable only with the defining class.
        const PRIVATE       = 0x0002;
        /// Declared `protectdd`; may be accessed within subclasses.
        const PROTECTED     = 0x0004;
        /// Declared `static`.
        const STATIC        = 0x0008;
        /// Declared `final`; no subclasses allowed.
        const FINAL         = 0x0010;
        /// Declared `volatile`; cannot be cached.
        const VOLATILE      = 0x0040;
        /// Declared `transient`; not written or read by a persistent object manager.
        const TRANSIENT     = 0x0080;
        /// Declared synthetic; not present in the source code.
        const SYNTHETIC     = 0x1000;
        /// Declared as an enum type.
        const ENUM          = 0x4000;
    }
}

impl Flags {
    pub(crate) fn read(r: &mut impl Read) -> io::Result<Self> {
        Ok(Self::from_bits_truncate(read_u2(r)?))
    }
}

#[derive(Clone, Debug)]
pub enum Constant {
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
}



/// [Java SE 7 &sect; 4.5](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.5):  field_info
#[derive(Clone, Debug)]
pub struct Field {
    pub flags:      Flags,
    pub name:       String,
    pub descriptor: String,
    pub deprecated: bool,
    pub constant:   Option<Constant>,
    _incomplete:    (),
}

impl Field {
    pub fn new(flags: Flags, name: String, descriptor: String) -> Self {
        Self {
            flags,
            name,
            descriptor,
            deprecated: false,
            constant: None,
            _incomplete: (),
        }
    }

    pub fn is_public(&self)     -> bool { self.flags.contains(Flags::PUBLIC) }
    pub fn is_private(&self)    -> bool { self.flags.contains(Flags::PRIVATE) }
    pub fn is_protected(&self)  -> bool { self.flags.contains(Flags::PROTECTED) }
    pub fn is_static(&self)     -> bool { self.flags.contains(Flags::STATIC) }
    pub fn is_final(&self)      -> bool { self.flags.contains(Flags::FINAL) }
    pub fn is_volatile(&self)   -> bool { self.flags.contains(Flags::VOLATILE) }
    pub fn is_transient(&self)  -> bool { self.flags.contains(Flags::TRANSIENT) }
    pub fn is_synthetic(&self)  -> bool { self.flags.contains(Flags::SYNTHETIC) }
    pub fn is_enum(&self)       -> bool { self.flags.contains(Flags::ENUM) }

    pub fn is_constant(&self)   -> bool { self.is_final() && self.is_static() && self.constant.is_some() }

    pub fn access(&self) -> Option<&'static str> {
        if      self.is_private()   { Some("private") }
        else if self.is_protected() { Some("protected") }
        else if self.is_public()    { Some("public") }
        else                        { None }
    }

    pub(crate) fn read_one(read: &mut impl Read, constants: &Constants) -> io::Result<Self> {
        let flags               = Flags::read(read)?;
        let name                = constants.get_utf8(read_u2(read)?)?.to_owned();
        let descriptor          = constants.get_utf8(read_u2(read)?)?.to_owned();
        let attributes_count    = read_u2(read)? as usize;

        let mut deprecated      = false;
        let mut constant        = None;
        for _ in 0..attributes_count {
            match Attribute::read(read, constants)? {
                Attribute::Deprecated { .. }            => { deprecated = true; },
                Attribute::ConstantValue_Integer(value) => { constant = Some(Constant::Integer(value)); },
                Attribute::ConstantValue_Long   (value) => { constant = Some(Constant::Long(value)); },
                Attribute::ConstantValue_Float  (value) => { constant = Some(Constant::Float(value)); },
                Attribute::ConstantValue_Double (value) => { constant = Some(Constant::Double(value)); },
                Attribute::ConstantValue_String (value) => { constant = Some(Constant::String(value)); },
                _ => {},
            }
        }

        Ok(Self{
            flags,
            name,
            descriptor,
            deprecated,
            constant,
            _incomplete: (),
        })
    }

    pub(crate) fn read_list(read: &mut impl Read, constants: &Constants) -> io::Result<Vec<Self>> {
        let n = read_u2(read)? as usize;
        let mut fields = Vec::with_capacity(n);
        for _ in 0..n {
            fields.push(Self::read_one(read, constants)?);
        }
        Ok(fields)
    }
}