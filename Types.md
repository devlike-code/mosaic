
/*
    f = λx: i32 -> x + 1            Typeof(f) : i32 -> i32          i32 -> T -> T
    h = λy: f32 -> x + 1.0          Typeof(h) : f32 -> f32          f32 -> T -> T
    g = ΛT -> λt: T -> t + 1        Typeof(g) : Type -> T -> T

    g(i32) = λt: i32 -> t + 1
    g(f32) = λt: f32 -> t + 1

    Indirection : ΛT -> Indirection`T
*/

/*

eliminate<T>(t: T) -> Result<(), Any>
eliminate(7) -> Ok(())
eliminate([ 1, 2, 3 ]) -> eliminate(1) -> Ok(())
eliminate(struct { x: 5, y: 7 }) -> eliminate(5).andThen(eliminate(7)) -> Ok(())


pravila eliminacije:
    0. uvedeni generik T - se moze unistiti ako je prethodno definisan
    1. prosti tipovi se samo uniste
    2. niz [ a, b, c ] - se moze unistiti samo ako se tip elemenata niza moze unistiti
    3. struct { a, b, c, ... } - se moze unistiti samo ako se svaki deo a, b, c,... moze unistiti
    4. enum { a, b, c, ... } - se moze unistiti samo ako se svaki deo a, b, c,... moze unistiti
    5. *<T> { ... } - se moze unistiti samo ako se T moze unistiti

1. impl T1 for T2 { ... }
    - T1 mora da postoji
    - T2 mora da postoji

2. impl<NT1> T1 for T2 { ... }
    - uvodimo novi tip NT1, kao generik
    - pri proveravanju T1, dobijemo da je T1 = NT1, i posto NT1 postoji - sve je ok
*/