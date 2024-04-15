//! Adapted from https://github.com/llogiq/pathsep

#[cfg(not(windows))]
#[macro_export]
macro_rules! pathsep {
    {} => { "/" }
}

#[cfg(windows)]
#[macro_export]
macro_rules! pathsep {
    {} => { "\\" }
}

#[macro_export]
macro_rules! path {
    () => { "" };
    (/) => { $crate::pathsep!() };
    (/ $($s:expr),*) => { concat!(path!(/), path!($($s),*)) };
    ($first:expr) => { $first };
    ($first:expr, $( $s:expr ),*) => {
        concat!($first, path!(/), path!($($s),*))
    };
}
