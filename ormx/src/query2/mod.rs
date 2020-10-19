use sqlx::query::Map;
use sqlx::{Database, Executor, IntoArguments};
use futures::stream::BoxStream;

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
/// Also, the number of conditions per query is currently limited to 4.
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
    ( $($t:tt)* ) => ( $crate::__build_query!((;;;); $($t)*) );
}

// create all branches necessary for the query, returning
// return = branch,*
// branch = (pat = expr),*; literal*; expr*
#[doc(hidden)]
#[macro_export]
macro_rules! __build_query {
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
    (
      @out
      $out:path,
      $($v:ident),*;
      ;
    ) => {};
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
      $out:path,
    ) => {{
        #[allow(unreachable_code)]
        loop {
            $crate::__import_output_variants!($(($($app=$ape),*;$($aq),*;$($ae),*;)),*);
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
      $out:path,
      $q:literal $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
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
        $($app:pat = $ape:expr),*;
        $($aq:literal),*;
        $($ae:expr),*;
      )),*;
      $out:path,
      ?($e:expr) $($t:tt)*
    ) => {
        $crate::__build_query! {
            $((
                $($app = $ape),*;
                $($aq,)* "?";
                $($ae,)* $e;
            )),*;
            $out,
            $($t)*
        }
    };
    // When encountering a branch, clone all existing branches, adding the new one.
    // [branches] + branch = [branches] + [branches + branch]
    (
      $((
        $($app:pat = $ape:expr),*;
        $($aq:literal),*;
        $($ae:expr),*;
      )),*;
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

// push multiple branch predicates, query fragments and parameters to all branches
#[doc(hidden)]
#[macro_export]
macro_rules! __push_fragments {
    (
        $(($ ($t:tt)* )),*;
        $out:path,
        ;
        ;
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__build_query! {
            $(( $($t)* )),* $($y)*;
            $out,
            $($x)*
        }
    };
    (
      $((
        $($app:pat = $ape:expr),*;
        $($aq:literal),*;
        $($ae:expr),*;
      )),*;
      $out:path,
      $qf:literal $(, $qo:literal)*;
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
            $out,
            $($qo),*;
            $($qe),*;
            [$($y)*];
            $($x)*
        }
    };
    (
        $((
            $($app:pat = $ape:expr),*;
            $($aq:literal),*;
            $($ae:expr),*;
        )),*;
        $out:path,
        $($qq:literal),*;
        $qf:expr $(, $qo:expr),*;
        [$($y:tt)*];
        $($x:tt)*
    ) => {
        $crate::__push_fragments! {
            $((
                $($app = $ape),*;
                $($aq,)* "?";
                $($ae,)* $qf;
            )),*;
            $out,
            $($qq),*;
            $($qo),*;
            [$($y)*];
            $($x)*
        }
    };
}

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

#[doc(hidden)]
#[macro_export]
#[rustfmt::skip]
macro_rules! __import_output_variants {
    (($($a:tt)*)) => (use $crate::exports::ConditionalMap1::*;);
    (($($a:tt)*), ($($b:tt)*)) => (use $crate::exports::ConditionalMap2::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*)) => (use $crate::exports::ConditionalMap4::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*), ($($e:tt)*), ($($f:tt)*), ($($g:tt)*), ($($h:tt)*)) => (use $crate::exports::ConditionalMap8::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*), ($($e:tt)*), ($($f:tt)*), ($($g:tt)*), ($($h:tt)*),
    ($($i:tt)*), ($($j:tt)*), ($($k:tt)*), ($($l:tt)*), ($($m:tt)*), ($($n:tt)*), ($($o:tt)*), ($($p:tt)*)) => (use $crate::exports::ConditionalMap16::*;);
}

macro_rules! make_conditional_map_ty {
    ($i:ident: $($fi:ident: $ff:ident, $fa:ident),*) => {
        pub enum $i<'q, DB, O, $($ff, $fa,)*>
        where
            DB: Database,
            O: Send + Unpin,
            $(
                $ff: Send + Sync + Fn(DB::Row) -> sqlx::Result<O>,
                $fa: 'q + Send + IntoArguments<'q, DB>,
            )*
        {
            $($fi(Map<'q, DB, $ff, $fa>)),*
        }
        impl<'q, DB, O, $($ff, $fa,)*> $i<'q, DB, O, $($ff, $fa,)*>
        where
            DB: Database,
            O: Send + Unpin,
            $(
                $ff: Send + Sync + Fn(DB::Row) -> sqlx::Result<O>,
                $fa: 'q + Send + IntoArguments<'q, DB>,
            )*
        {
            pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, sqlx::Result<O>>
            where
                'q: 'e,
                E: 'e + Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                $($ff: 'e),*
            {
                match self { $(
                    Self::$fi(x) => x.fetch(executor)
                ),* }
            }
            pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<Vec<O>>
            where
                'q: 'e,
                DB: 'e,
                E: 'e + Executor<'c, Database = DB>,
                O: 'e
            {
               match self { $(
                        Self::$fi(x) => x.fetch_all(executor).await
               ),* }
            }
            pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<O>
            where
                'q: 'e,
                E: 'e + Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
            {
                match self { $(
                    Self::$fi(x) => x.fetch_one(executor).await
                ),* }
            }
            pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<Option<O>>
            where
                'q: 'e,
                E: 'e + Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
            {
                match self { $(
                    Self::$fi(x) => x.fetch_optional(executor).await
                ),* }
            }
        }
    };
}

// When constructing a conditional query, the number of branches is 2^(number of patterns), since
// every pattern doubles the number of branches. Let's support 16 branches, equal to 4 patterns per
// query, for now.
#[rustfmt::skip] make_conditional_map_ty!(ConditionalMap1: _1: F1, A1);
#[rustfmt::skip] make_conditional_map_ty!(ConditionalMap2: _1: F1, A1, _2: F2, A2);
#[rustfmt::skip] make_conditional_map_ty!(ConditionalMap4: _1: F1, A1, _2: F2, A2, _3: F3, A3, _4: F4, A4);
#[rustfmt::skip] make_conditional_map_ty!(ConditionalMap8: _1: F1, A1, _2: F2, A2, _3: F3, A3, _4: F4, A4, _5: F5, A5, _6: F6, A6, _7: F7, A7, _8: F8, A8);
#[rustfmt::skip] make_conditional_map_ty!(ConditionalMap16: _1: F1, A1, _2: F2, A2, _3: F3, A3, _4: F4, A4, _5: F5, A5, _6: F6, A6, _7: F7, A7, _8: F8, A8, _9: F9, A9, _10: F10, A10, _11: F11, A11, _12: F12, A12, _13: F13, A13, _14: F14, A14, _15: F15, A15, _16: F16, A16);

