#[cfg(test)]
mod tests {

    #[test]
    fn test_detect_peaks() {
        let input =  vec![(4.0, 0.0), (6.0, 0.0), (5.0, 0.0), (1.0, 0.0), (3.0, 0.0), (2.0, 0.0)];
        assert_eq!(Vec::<usize>::new(), crate::detect_peaks(10.0, input.clone()));
        assert_eq!(vec![1], crate::detect_peaks(5.0, input.clone()));
        assert_eq!(vec![1, 4], crate::detect_peaks(2.0, input.clone()));
    }

    #[test]
    fn test_linear_interpolation() {
        assert_eq!(2.0, crate::linear_interpoliation((1.0, 1.0), (3.0, 3.0), 2.0));
    }
}
