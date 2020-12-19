use crate::models;
use std::io::{Cursor, Error as IoError, Read, Write};
use std::{str, u8};
use uuid::Uuid;

pub enum Component<'a> {
    Uuid(Uuid),
    UnsizedString(&'a str),
    Type(&'a models::Type),
}

impl<'a> Component<'a> {
    fn len(&self) -> usize {
        match *self {
            Component::Uuid(_) => 16,
            Component::UnsizedString(s) => s.len(),
            Component::Type(t) => t.0.len() + 1,
        }
    }

    fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), IoError> {
        match *self {
            Component::Uuid(uuid) => {
                cursor.write_all(uuid.as_bytes())?;
            }
            Component::UnsizedString(s) => {
                cursor.write_all(s.as_bytes())?;
            }
            Component::Type(t) => {
                cursor.write_all(&[t.0.len() as u8])?;
                cursor.write_all(t.0.as_bytes())?;
            }
        };

        Ok(())
    }
}

pub fn build(components: &[Component]) -> Vec<u8> {
    let len = components.iter().fold(0, |len, component| len + component.len());
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(len));

    for component in components {
        if let Err(err) = component.write(&mut cursor) {
            panic!("Could not write bytes: {}", err);
        }
    }

    cursor.into_inner()
}

pub fn read_uuid<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> Uuid {
    let mut buf: [u8; 16] = [0; 16];
    cursor.read_exact(&mut buf).unwrap();
    Uuid::from_slice(&buf).unwrap()
}

pub fn read_type<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> models::Type {
    let t_len = {
        let mut buf: [u8; 1] = [0; 1];
        cursor.read_exact(&mut buf).unwrap();
        buf[0] as usize
    };

    let mut buf = vec![0u8; t_len];
    cursor.read_exact(&mut buf).unwrap();

    unsafe {
        let s = str::from_utf8_unchecked(&buf).to_string();
        models::Type::new_unchecked(s)
    }
}

pub fn read_unsized_string<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> String {
    let mut buf = String::new();
    cursor.read_to_string(&mut buf).unwrap();
    buf
}
