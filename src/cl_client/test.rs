use crate::cl_client::client::{get_value_input, parse_vec};

#[test]
pub fn test_input() {
    // oh wait, I can't test input
    // let n = get_value_input(|&i| 0 < i && i < 3);
    // println!("{}", n);
}

#[test]
fn test_parse_vec () {
    println!("printed vec: {:?}", vec![1, 2, 3]);
    let vec: Result<Vec<u32>, _> = parse_vec("1, 2,3,  4   , 5,6");
    println!("parsed vec: {:?}", vec);
}