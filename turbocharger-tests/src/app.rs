#![allow(non_snake_case)]
#![cfg_attr(feature = "wasm", allow(dead_code))]

use turbocharger::prelude::*;

#[backend(js)]
pub async fn run_test() -> i32 {
	println!("in run_test");
	42
}

#[backend(js)]
pub async fn one_hundred() -> i32 {
	connection_local!(one_hundred: &mut i32);
	if *one_hundred == 0 {
		*one_hundred = 100;
	}
	*one_hundred
}

#[backend(js)]
pub async fn two_hundred() -> i32 {
	connection_local!(two_hundred: &mut i32);
	if *two_hundred == 0 {
		*two_hundred = 200;
	}
	*two_hundred
}

#[backend(js)]
pub async fn two_hundred_increment() -> i32 {
	connection_local!(two_hundred: &mut i32);
	*two_hundred += 1;
	*two_hundred
}
