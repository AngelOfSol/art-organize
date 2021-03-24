pub fn next<T, I>(mut iter: I, value: T) -> Option<T>
where
    T: PartialEq,
    I: Iterator<Item = T> + Clone,
{
    iter.clone()
        .position(|x| x == value)
        .and_then(move |idx| iter.nth(idx + 1))
}

pub fn prev<T, I>(mut iter: I, value: T) -> Option<T>
where
    T: PartialEq,
    I: Iterator<Item = T> + Clone,
{
    iter.clone()
        .position(|x| x == value)
        .and_then(move |idx| idx.checked_sub(1))
        .and_then(|idx| iter.nth(idx))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next() {
        let x = vec![1, 2, 3, 4, 5];

        assert_eq!(next(x.iter(), &3), Some(&4));
        assert_eq!(next(x.iter(), &5), None);
    }

    #[test]
    fn test_prev() {
        let x = vec![1, 2, 3, 4, 5];

        assert_eq!(prev(x.iter(), &3), Some(&2));
        assert_eq!(prev(x.iter(), &1), None);
    }
}
