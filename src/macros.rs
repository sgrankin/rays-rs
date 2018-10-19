#[macro_export]
macro_rules! iff {
    ($x: expr, $y: expr, $z: expr) => {
        if $x {
            $y
        } else {
            $z
        }
    };
}

#[macro_export]
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $y: expr) => ({
        let x = $x;
        let y = $y;
        iff!(x < y, x, y)
        });
    ($x: expr, $($xs: expr), +) => (min!($x, min!($($xs), +)));
}

#[macro_export]
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $y: expr) => ({
        let x = $x;
        let y = $y;
        iff!(x > y, x, y)
        });
    ($x: expr, $($xs: expr), +) => ( max!($x, max!($($xs), +)));
}
