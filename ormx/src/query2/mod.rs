#[doc(hidden)]
#[rustfmt::skip]
pub mod map;
#[cfg(any(feature = "mysql", feature = "mariadb"))]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

/// An improved version of `sqlx::query_as!`.
///
/// # Syntax
/// The syntax of `conditional_query_as!` differs from the original `sqlx::query_as!`.
/// Formally, it accepts this syntax:
/// ```rust,ignore
/// conditional_query_as!(
///     PATH,
///     (
///         LITERAL |
///         ?(EXPRESSION) |
///         PATTERN = EXPRESSION => { (LITERAL | ?(EXPRESSION))* }
///     )*
/// )
/// ```
/// Arguments are now provided inline with the query like this:
/// ```rust,ignore
/// let user_id = Some(2);
/// conditional_query_as!(
///     User,
///     "SELECT * FROM users WHERE user_id =" ?(user_id.unwrap())
/// );
/// ```
/// Also, `conditional_query_as!` can parse multiple string literals like this:
/// ```rust,ignore
/// conditional_query_as!(
///     User,
///     "SELECT * FROM users"
///     "WHERE user_id =" ?(user_id)
///     "AND first_name =" ?(first_name)
/// );
/// ```
///
/// # Conditions
/// `conditional_query_as!` can be used to have queries depend on a condition during runtime.
/// This is achieved by checking the correctness of all possible queries at compile time.
/// The syntax for a condition is
/// ```ignore
/// PATTERN = EXPRESSION => { (LITERAL | ?(EXPRESSION))* }
/// ```
/// Please note that conditions can't be nested right now.
/// Also, the number of conditions per query is currently limited to 5.
///
/// Example:
/// ```rust,ignore
/// let limit = Some(10);
/// conditional_query_as!(
///     User,
///     "SELECT * FROM users"
///     Some(l) = limit => {
///         "LIMIT" ?(l)
///     }
/// );
/// ```
///
#[macro_export]
macro_rules! conditional_query_as {
    ( $($t:tt)* ) => {
        $crate::__conditional_query_as_impl!($($t)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __build_query {
    // --- @out_branch ---
    // Build one branch
    (
        @out_branch
        $out:path,
        $variant:ident;
        $appf:pat = $apef:expr $(, $appo:pat = $apeo:expr)*;
        $($aq:literal),*;
        $($ae:expr),*;
    ) => {
        if let $appf = $apef {
            $crate::__build_query! {
                @out_branch
                $out,
                $variant;
                $($appo = $apeo),*;
                $($aq),*;
                $($ae),*;
            }
        }
    };
    (
        @out_branch
        $out:path,
        $variant:ident;
        ;
        $($aq:literal),*;
        $($ae:expr),*;
    ) => {
        break $variant($crate::__sqlx_query_as!(
            $out,
            $($aq),*;
            $($ae),*
        ));
    };

    // --- @out ---
    // Build branches, one by one
    ( @out $out:path, $($v:ident),*; ; ) => {};
    (
        @out
        $out:path,
        $vf:ident $(, $vo:ident)*;
        (
            $($appf:pat = $apef:expr),*;
            $($aqf:literal),*;
            $($aef:expr),*;
        )
        $(, (
            $($appo:pat = $apeo:expr),*;
            $($aqo:literal),*;
            $($aeo:expr),*;
        ))*;
    ) => {{
        $crate::__build_query!(
            @out_branch
            $out,
            $vf;
            $($appf = $apef),*;
            $($aqf),*;
            $($aef),*;
        );
        $crate::__build_query!(
            @out
            $out,
            $($vo),*;
            $((
                $($appo = $apeo),*;
                $($aqo),*;
                $($aeo),*;
            )),*;
        );
    }};

    // -- Main Macro --
    (
        $((
            $($bp:literal),*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
    ) => {{
        #[allow(unreachable_code)]
        // TODO: remove this once sqlx 0.4 is out
        #[allow(clippy::toplevel_ref_arg)]
        loop {
            $crate::__import_conditional_map!($(($($app=$ape),*;$($aq),*;$($ae),*;)),*);
            $crate::__build_query!(
                @out
                $out,
                _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16;
                $((
                    $($app = $ape),*;
                    $($aq),*;
                    $($ae),*;
                )),*;
            );
            unreachable!();
        }
    }};
    // When encountering a query string, add it to all branches
    (
        $((
            $($bp:literal),*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        $q:literal $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
                $($bp),*;
                $($app = $ape),*;
                $($aq,)* $q;
                $($ae),*;
            )),*;
            $out,
            $($t)*
        }
    };
    // When encountering a query parameter, add it to all branches
    (
        $((
            $bpf:literal $(, $bpo:literal)*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        ?($e:expr) $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
                $($bpo),*;
                $($app = $ape),*;
                $($aq,)* $bpf;
                $($ae,)* $e;
            )),*;
            $out,
            $($t)*
        }
    };
    // When encountering a branch, clone all existing branches, adding the new one.
    (
        $((
            $($bp:literal),*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        $bpp:pat = $bpe:expr => {
            $($bt:tt)*
        }
        $($t:tt)*
    ) => {
        $crate::__push_fragments!(
            // all existing branches
            $((
                $($bp),*;
                $($app = $ape,)* $bpp = $bpe;
                $($aq),*;
                $($ae),*;
            )),*;
            // out path
            $out,
            // content of new branch
            { $($bt)* };

            // copy of old branches
            [$(, (
                $($bp),*;
                $($app = $ape),*;
                $($aq),*;
                $($ae),*;
            ))*];
            // remaining tokens
            $($t)*
        )
    };
}

// Push multiple branch predicates, query fragments and parameters to all branches.
#[doc(hidden)]
#[macro_export]
macro_rules! __push_fragments {
    // We are done!
    (
        $(($ ($t:tt)* )),*;
        $out:path,
        {};
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__build_query! {
            $(( $($t)* )),* $($y)*;
            $out,
            $($x)*
        }
    };
    // When we encounter a literal, add it to all branches.
    (
        $((
            $($bp:literal),*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        { $qf:literal $($bo:tt)* }; // <- the literal
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__push_fragments!{
            $((
                $($bp),*;
                $($app = $ape),*;
                $($aq,)* $qf;
                $($ae),*;
            )),*;
            $out,
            { $($bo)* };
            [$($y)*];
            $($x)*
        }
    };
    // When we encounter an argument, add it to all branches.
    (
        $((
            $bpf:literal $(, $bpo:literal)*;
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        { ?($qf:expr) $($bo:tt)* }; // <- the argument
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__push_fragments! {
            $((
                $($bpo),*;
                $($app = $ape),*;
                $($aq,)* $bpf;
                $($ae,)* $qf;
            )),*;
            $out,
            { $($bo)* };
            [$($y)*];
            $($x)*
        }
    };
}

// Do the actual call to sqlx::query_as!
#[doc(hidden)]
#[macro_export]
macro_rules! __sqlx_query_as {
    (
        $out:path,
        $f:literal $(, $o:literal)*;
        $($e:expr),*
    ) => {
        sqlx::query_as!($out, $f $(+ " " + $o)*, $($e),*)
    };
}
