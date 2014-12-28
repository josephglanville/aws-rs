#![crate_name = "aws"]
#![crate_type = "lib"]

extern crate curl;
#[cfg(unix)] extern crate openssl;
extern crate serialize;

pub mod glacier;

#[cfg (test)]
mod test;