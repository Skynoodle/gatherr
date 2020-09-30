use std::iter::FromIterator;
struct Gatherr<T, E>(Result<T, E>);

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

pub trait IterExt<A, B>: IntoIterator<Item = Result<A, B>> + Sized {
    fn gatherr<T: FromIterator<A>, E: FromIterator<B>>(self) -> Result<T, E> {
        let Gatherr(result) = self.into_iter().collect();
        result
    }
}

impl<A, B, I: IntoIterator<Item = Result<A, B>> + Sized> IterExt<A, B> for I {}

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
        let result: Result<Vec<_>, Vec<_>> = v.gatherr();

        assert_eq!(&result.unwrap(), &["Hello", "World", "!"]);
    }
    #[test]
    fn err_gather() {
        let v: Vec<Result<String, _>> = vec![
            Err("Goodbye".to_owned()),
            Err("cruel".to_owned()),
            Err("world".to_owned()),
        ];
        let result: Result<Vec<_>, Vec<_>> = v.gatherr();

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
        let result: Result<Vec<_>, Vec<_>> = v.gatherr();

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
        let result: Result<Vec<_>, Vec<_>> = v.gatherr();

        assert_eq!(&result.unwrap_err(), &["Goodbye", "cruel", "world"]);
    }

    #[test]
    fn empty_gather() {
        let result: Result<Vec<String>, Vec<String>> = std::iter::empty().gatherr();
        assert_eq!(result, Ok(Vec::new()));
    }
}
