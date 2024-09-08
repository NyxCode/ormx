use futures::Stream;
use sqlx::{Database, Executor, IntoArguments, Error, query::Map};

#[doc(hidden)]
#[macro_export]
macro_rules! __import_conditional_map {
    (($($a:tt)*)) => (use $crate::exports::ConditionalMap1::*;);
    (($($a:tt)*), ($($b:tt)*)) => (use $crate::exports::ConditionalMap2::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*)) => (use $crate::exports::ConditionalMap4::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*),
     ($($e:tt)*), ($($f:tt)*), ($($g:tt)*), ($($h:tt)*)) => (use $crate::exports::ConditionalMap8::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*),
     ($($e:tt)*), ($($f:tt)*), ($($g:tt)*), ($($h:tt)*),
     ($($i:tt)*), ($($j:tt)*), ($($k:tt)*), ($($l:tt)*),
     ($($m:tt)*), ($($n:tt)*), ($($o:tt)*), ($($p:tt)*)) => (use $crate::exports::ConditionalMap16::*;);
    (($($a:tt)*), ($($b:tt)*), ($($c:tt)*), ($($d:tt)*),
     ($($e:tt)*), ($($f:tt)*), ($($g:tt)*), ($($h:tt)*),
     ($($i:tt)*), ($($j:tt)*), ($($k:tt)*), ($($l:tt)*),
     ($($m:tt)*), ($($n:tt)*), ($($o:tt)*), ($($p:tt)*),
     ($($q:tt)*), ($($r:tt)*), ($($s:tt)*), ($($t:tt)*),
     ($($u:tt)*), ($($v:tt)*), ($($w:tt)*), ($($x:tt)*),
     ($($y:tt)*), ($($z:tt)*), ($($a1:tt)*), ($($b1:tt)*),
     ($($b2:tt)*), ($($b3:tt)*), ($($b4:tt)*), ($($b5:tt)*)) => (use $crate::exports::ConditionalMap32::*;);
}

macro_rules! make_conditional_map_ty {
    ($i:ident: $($fi:ident: $ff:ident, $fa:ident),*) => {
        pub enum $i<'q, DB, O, $($ff, $fa,)*>
        where
            DB: Database,
            O: Send + Unpin,
            $(
                $ff: FnMut(DB::Row) -> Result<O, Error> + Send,
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
                $ff: FnMut(DB::Row) -> Result<O, Error> + Send,
                $fa: 'q + Send + IntoArguments<'q, DB>,
            )*
        {
            pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> impl Stream<Item = sqlx::Result<O>> + Send + Unpin + 'e
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
// every pattern doubles the number of branches. Let's support 32 branches, equal to 5 patterns per
// query, for now.
make_conditional_map_ty!(ConditionalMap1: _1: F1, A1);
make_conditional_map_ty!(ConditionalMap2: _1: F1, A1, _2: F2, A2);
make_conditional_map_ty!(ConditionalMap4:
    _1: F1, A1, _2: F2, A2, _3: F3, A3, _4: F4, A4
);
make_conditional_map_ty!(ConditionalMap8:
    _1: F1, A1, _2: F2, A2, _3: F3, A3, _4: F4, A4,
    _5: F5, A5, _6: F6, A6, _7: F7, A7, _8: F8, A8
);
make_conditional_map_ty!(ConditionalMap16:
    _1:  F1,  A1,  _2:  F2,  A2,  _3:  F3,  A3,  _4:  F4,  A4,
    _5:  F5,  A5,  _6:  F6,  A6,  _7:  F7,  A7,  _8:  F8,  A8,
    _9:  F9,  A9,  _10: F10, A10, _11: F11, A11, _12: F12, A12,
    _13: F13, A13, _14: F14, A14, _15: F15, A15, _16: F16, A16
);
make_conditional_map_ty!(ConditionalMap32:
    _1:  F1,  A1,  _2:  F2,  A2,  _3:  F3,  A3,  _4:  F4,  A4,
    _5:  F5,  A5,  _6:  F6,  A6,  _7:  F7,  A7,  _8:  F8,  A8,
    _9:  F9,  A9,  _10: F10, A10, _11: F11, A11, _12: F12, A12,
    _13: F13, A13, _14: F14, A14, _15: F15, A15, _16: F16, A16,
    _17: F17, A17, _18: F18, A18, _19: F19, A19, _20: F20, A20,
    _21: F21, A21, _22: F22, A22, _23: F23, A23, _24: F24, A24,
    _25: F25, A25, _26: F26, A26, _27: F27, A27, _28: F28, A28,
    _29: F29, A29, _30: F30, A30, _31: F31, A31, _32: F32, A132
);
