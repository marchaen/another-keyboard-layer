#[no_mangle]
pub extern "C" fn triple(number: i32) -> i32 {
    number * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = triple(2);
        assert_eq!(result, 6);
    }
}
