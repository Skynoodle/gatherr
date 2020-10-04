use std::iter::FromIterator;

/// A newtype implementing FromIterator to collect into a result preserving all
/// error values, instead of just the first as `FromIterator` for `Result` does
///
/// ```
/// # use gatherr::Gatherr;
/// let v = vec![Ok("a"), Err(1), Ok("b"), Err(2)];
///
/// let Gatherr(result): Gatherr<Vec<&str>, Vec<u32>>
///     = v.into_iter().collect();
///
/// assert_eq!(result, Err(vec![1, 2]));
/// ```
///
/// Using this directly can be awkward due to the necessary additional type
/// annotation. Consider using [the extension trait method](trait.IterExt.html#method.gatherr)
/// or the [freestanding gatherr function](fn.gatherr.html) instead
pub struct Gatherr<T, E>(pub Result<T, E>);

impl<A, B, T: FromIterator<A>, E: FromIterator<B>> FromIterator<Result<A, B>> for Gatherr<T, E> {
    fn from_iter<I: IntoIterator<Item = Result<A, B>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut first_err = None;
        let ok = (&mut iter)
            .scan((), |_, i| match i {
                Ok(v) => Some(v),
                Err(e) => {
                    first_err = Some(e);
                    None
                }
            })
            .collect();
        Gatherr(if let Some(first_err) = first_err {
            drop(ok);
            Err(std::iter::once(first_err)
                .chain(iter.filter_map(|r| r.err()))
                .collect())
        } else {
            Ok(ok)
        })
    }
}


/// An extension trait for iterators of `Result`s to easily collect without the
/// extra newtype
pub trait IterExt<A, B>: Iterator<Item = Result<A, B>> + Sized {
    /// Collect all Ok or Err values from this iterator into a single `Result`
    // of collections
    ///
    /// ```
    /// use gatherr::IterExt;
    /// let v = vec![Ok("a"), Err(1), Ok("b"), Err(2)];
    ///
    /// let result: Result<Vec<&str>, Vec<u32>> = v.into_iter().gatherr();
    ///
    /// assert_eq!(result, Err(vec![1, 2]));
    /// ```
    fn gatherr<T: FromIterator<A>, E: FromIterator<B>>(self) -> Result<T, E> {
        let Gatherr(result) = self.collect();
        result
    }
}

impl<A, B, I: Iterator<Item = Result<A, B>> + Sized> IterExt<A, B> for I {}

/// Collect all Ok or Err values from an iterator into a single `Result` of
/// collections
///
/// ```
/// # use gatherr::gatherr;
/// let v = vec![Ok("a"), Err(1), Ok("b"), Err(2)];
///
/// let result: Result<Vec<&str>, Vec<u32>> = gatherr(v);
///
/// assert_eq!(result, Err(vec![1, 2]));
/// ```
pub fn gatherr<
    A,
    B,
    T: FromIterator<A>,
    E: FromIterator<B>,
    I: IntoIterator<Item = Result<A, B>>,
>(
    iter: I,
) -> Result<T, E> {
    let Gatherr(result) = iter.into_iter().collect();
    result
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ok_gather() {
        let v: Vec<Result<_, String>> = vec![
            Ok("Hello".to_owned()),
            Ok("World".to_owned()),
            Ok("!".to_owned()),
        ];
        let result: Result<Vec<_>, Vec<_>> = v.into_iter().gatherr();

        assert_eq!(&result.unwrap(), &["Hello", "World", "!"]);
    }
    #[test]
    fn err_gather() {
        let v: Vec<Result<String, _>> = vec![
            Err("Goodbye".to_owned()),
            Err("cruel".to_owned()),
            Err("world".to_owned()),
        ];
        let result: Result<Vec<_>, Vec<_>> = v.into_iter().gatherr();

        assert_eq!(&result.unwrap_err(), &["Goodbye", "cruel", "world"]);
    }
    #[test]
    fn mixed_gather_initial_ok() {
        let v: Vec<Result<String, _>> = vec![
            Ok("Hello".to_owned()),
            Ok("World".to_owned()),
            Err("Goodbye".to_owned()),
            Ok("!".to_owned()),
            Err("cruel".to_owned()),
            Err("world".to_owned()),
        ];
        let result: Result<Vec<_>, Vec<_>> = v.into_iter().gatherr();

        assert_eq!(&result.unwrap_err(), &["Goodbye", "cruel", "world"]);
    }
    #[test]
    fn mixed_gather_initial_err() {
        let v: Vec<Result<String, _>> = vec![
            Err("Goodbye".to_owned()),
            Ok("Hello".to_owned()),
            Err("cruel".to_owned()),
            Err("world".to_owned()),
            Ok("World".to_owned()),
            Ok("!".to_owned()),
        ];
        let result: Result<Vec<_>, Vec<_>> = v.into_iter().gatherr();

        assert_eq!(&result.unwrap_err(), &["Goodbye", "cruel", "world"]);
    }

    #[test]
    fn empty_gather() {
        let result: Result<Vec<String>, Vec<String>> = std::iter::empty().gatherr();
        assert_eq!(result, Ok(Vec::new()));
    }
}
