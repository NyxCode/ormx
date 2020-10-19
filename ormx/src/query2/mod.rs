#[doc(hidden)]
#[rustfmt::skip]
pub mod map;
#[cfg(feature = "mysql")]
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

/// Do the actual computation.
/// # Terms
/// - "Branch" - a possible query, given zero or more branch predicates
/// - "Branch Predicate" - a pattern which has to match an expression for the branch to be executed
/// - "Query Fragment" - part of a query string
/// - "Argument" - an argument used to execute a parameterized query
///
/// # How does it work?
/// This macro takes a list of all branches in this format:
/// ```ignore
/// (
///     $( (PATTERN = EXPRESSION) ),*; // branch predicates
///     $(LITERAL),*; // query fragments
///     $( (EXPRESSIONS) ),*; // arguments
/// )
/// ```
/// Then, it takes either a branch, a query fragment or an argument. In case of a query fragment or
/// an argument, it is added to all branches and the macro recurses with the remaining tokens.
/// When encountering a branch, all existing branches will be cloned and the content of the new
/// branch will be added.
/// It is important that the "old" branches are at the back of the branch list.
///
/// When done, this macro will product a loop which contains all branches.
/// This will look something like this:
/// ```ignore
/// loop {
///     // BRANCH 1
///     if PATTERN = EXPRESSION { // Branch Predicate
///         break sqlx::query_as!(..)
///     }
///     // Branch 2
///     if PATTERN = EXPRESSION { // Branch Predicate
///         break sqlx::query_as!(..)
///     }
/// }
/// ```
///
/// Because, however, each invocation of sqlx::query_as! results in a different type, the result
/// will be wrapped in [ConditionalMapX], where X is the count of all branches.
/// [ConditionalMapX] is just an enum which defers to its variants when calling methods like [fetch_one].
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
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $($phx:literal),*;
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
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $($phx:literal),*;
        $out:path,
        $q:literal $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
                $($app = $ape),*;
                $($aq,)* $q;
                $($ae),*;
            )),*;
            $($phx),*;
            $out,
            $($t)*
        }
    };
    // When encountering a query parameter, add it to all branches
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $ph:literal $(, $phx:literal)*;
        $out:path,
        ?($e:expr) $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
                $($app = $ape),*;
                $($aq,)* $ph;
                $($ae,)* $e;
            )),*;
            $($phx),*;
            $out,
            $($t)*
        }
    };
    // When encountering a branch, clone all existing branches, adding the new one.
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $($phx:literal),*;
        $out:path,
        $bpp:pat = $bpe:expr => { $($bq:literal $(?($be:expr))*)* }
        $($t:tt)*
    ) => {
        $crate::__push_fragments!(
            $((
                $($app = $ape,)* $bpp = $bpe;
                $($aq),*;
                $($ae),*;
            )),*;
            $($phx),*;
            $out,
            $($bq),*;
            $($($be),*),*;
            [$(, (
                $($app = $ape),*;
                $($aq),*;
                $($ae),*;
            ))*];
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
        $($phx:literal),*;
        $out:path,
        ;
        ;
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__build_query! {
            $(( $($t)* )),* $($y)*;
            $($phx),*;
            $out,
            $($x)*
        }
    };
    // When we encounter a literal, add it to all branches.
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $($phx:literal),*;
        $out:path,
        $qf:literal $(, $qo:literal)*; // <- the literal
        $($qe:expr),*;
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__push_fragments!{
            $((
                $($app = $ape),*;
                $($aq,)* $qf;
                $($ae),*;
            )),*;
            $($phx),*;
            $out,
            $($qo),*;
            $($qe),*;
            [$($y)*];
            $($x)*
        }
    };
    // When we encounter an argument, add it to all branches.
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $ph:literal $(, $phx:literal)*;
        $out:path,
        $($qq:literal),*;
        $qf:expr $(, $qo:expr),*; // <- the argument
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__push_fragments! {
            $((
                $($app = $ape),*;
                $($aq,)* $ph;
                $($ae,)* $qf;
            )),*;
            $($phx),*;
            $out,
            $($qq),*;
            $($qo),*;
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
