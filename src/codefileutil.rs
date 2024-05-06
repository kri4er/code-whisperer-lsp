/// Function to find first different caracter id from string
/// EG: left: 123, right :143 will return 1
/// EG: left: 123, right :1243 will return 3
pub fn find_carret_pos(left: &str, right: &str) -> usize {
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    let mut l: usize = 0;
    let mut r: usize = 0;
    for (lv, rv) in left.chars().zip(right.chars()) {
        if lv != rv {
            break;
        }
        l = l + 1;
        r = r + 1;
    }
    l
}

#[cfg(test)]
mod test_code_file_util {
    use std::usize;

    use crate::codefileutil::find_carret_pos;

    #[tokio::test]
    async fn test_same_strings_returns_string_size() {
        let left = "123";
        let right = "123";
        assert_eq!(left, right);
        assert_eq!(left.len(), find_carret_pos(left, right));
    }

    #[tokio::test]
    async fn test_different_strings_returns_0() {
        let left = "321";
        let right = "123";
        assert_eq!(0 as usize, find_carret_pos(left, right));
    }

    #[tokio::test]
    async fn test_partially_matched_strings_returns_index_where_they_split() {
        let left = "124";
        let right = "123";
        assert_eq!(2 as usize, find_carret_pos(left, right));
    }

    #[tokio::test]
    async fn test_partially_matched_strings_different_len_returns_index_where_they_split() {
        let left = "123";
        let right = "12345";
        assert_eq!(3 as usize, find_carret_pos(left, right));
    }
}
