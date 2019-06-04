//! Helper macros

#[cfg(test)]
#[macro_use]
mod test_macros {
    /// Construct new Url
    /// Will panic! if provided data cannot be parsed

    #[macro_export]
    macro_rules! url {
        ($it: expr) => {{
            use url::Url;

            Url::parse($it).unwrap()
        }};
    }

    /// Count passed arguments

    #[macro_export]
    macro_rules! count {
        ($cur: tt $(, $tail: tt)* $(,)*) => {
            1 + count!($($tail,)*)
        };

        () => { 0 };
    }

    /// Construct new HashSet, similar to vec![] macro

    #[macro_export]
    macro_rules! hashset {
        () => {{
            use hashbrown::HashSet;

            HashSet::new()
        }};

        ( $($value:expr),+ $(,)* ) => {{
            use hashbrown::HashSet;

            let capacity = count!($($value),+);

            let mut hash = HashSet::with_capacity(capacity);
            $(
                hash.insert($value);
            )*

            hash
        }};
    }
}
